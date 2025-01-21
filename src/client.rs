use std::time::Duration;
use reqwest::{header::{HeaderMap, HeaderValue}, redirect::Policy, ClientBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::error::{AjaxClientError, WikidotRespondError};

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36";

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct AjaxConfig{
    pub attempt_limit: i8,
    pub retry_interval: i8,
    pub semaphore_limit: i8,
    pub request_timeout: i8,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct AjaxClient{
    pub config: AjaxConfig,
    pub cookies: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AjaxResponse{
    pub status: String,
    pub body: String,
}

impl Default for AjaxConfig{
    fn default() -> Self{
        AjaxConfig{
            attempt_limit: 5,
            retry_interval: 5,
            semaphore_limit: 5,
            request_timeout: 60,
        }
    }
}

impl AjaxClient{
    pub fn new() -> Self{
        AjaxClient{
            config: AjaxConfig::default(),
            cookies: None,
        }
    }

    pub async fn from(username: &str, password: &str) -> Result<Self, AjaxClientError>{
        let params = [
            ("login", username),
            ("password", password),
            ("action", "Login2Action"),
            ("event", "login")
        ];
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_str(UA)?);
        let client = ClientBuilder::new()
            .redirect(Policy::none())
            .default_headers(headers)
            .build()?;
        let _response = client.post("https://www.wikidot.com/default--flow/login__LoginPopupScreen")
            .form(&params).send().await?;
        let mut cookies = String::new();
        for c in _response.cookies(){
            let c_name = c.name();
            let c_val = if c_name == "wikidot_token7" {"123456"} else {c.value()};
            cookies.push_str(&format!("{c_name}={c_val}; "));
        }
        Ok(AjaxClient{
            config: AjaxConfig::default(),
            cookies: Some(cookies),
        })
    }

    async fn process_post_response(value: Result<reqwest::Response, reqwest::Error>) -> Result<AjaxResponse, AjaxClientError>{
        let ajax = Self::process_response(value)?.json::<AjaxResponse>().await?;
        if &ajax.status == "try_again"{
            Err(WikidotRespondError::try_again())?
        }
        else if &ajax.body == "" {
            Err(WikidotRespondError::empty())?
        }
        else {
            Ok(ajax)
        }
    }

    fn process_response(value: Result<reqwest::Response, reqwest::Error>) -> Result<Response, AjaxClientError>{
        let response = value?;

        let status = response.status();
        match status{
            StatusCode::REQUEST_TIMEOUT | StatusCode::BAD_GATEWAY => Err(WikidotRespondError::status(status))?,
            _ => {
                Ok(response)
            }
        }
    }

    pub async fn client(&self) -> Result<reqwest::Client, AjaxClientError>{
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_str(UA)?);
        if let Some(cookies) = &self.cookies {
            headers.insert("cookie", HeaderValue::from_str(&cookies)?);
        }
        else {
            headers.insert("cookie", HeaderValue::from_str("wikidot_token7=123456")?);
        }
        Ok(ClientBuilder::new()
            .redirect(Policy::none())
            .timeout(Duration::from_secs(self.config.request_timeout as u64))
            .cookie_store(true)
            .default_headers(headers)
            .build()?)
    }
    
    pub async fn request(&self, param: &[(&str, &str)], url: &str) -> Result<AjaxResponse, AjaxClientError>{
        let mut attempt: i8 = 0;
        let mut param_vec = Vec::from([
            ("callbackIndex", "0"), 
            ("wikidot_token7", "123456")
        ]);
        param_vec.extend_from_slice(param);
        loop {
            let response = self.client().await?.post(url)
                .form(param_vec.as_slice())
                .send().await;

            let processed = Self::process_post_response(response).await;

            if processed.is_ok() || attempt > self.config.attempt_limit {
                return processed
            }

            attempt += 1;
            sleep(Duration::from_secs(self.config.retry_interval as u64)).await;
        }
    }

    pub async fn get(&self, url: &str) -> Result<Response, AjaxClientError>{
        let mut attempt: i8 = 0;
        loop {
            let response = self.client().await?.get(url).send().await;
            let processed = Self::process_response(response);

            if processed.is_ok() || attempt > self.config.attempt_limit {
                return processed
            }

            attempt += 1;
            sleep(Duration::from_secs(self.config.retry_interval as u64)).await;
        }
    }
}