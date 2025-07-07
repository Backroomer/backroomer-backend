use std::sync::Arc;
use regex::Regex;
use reqwest::StatusCode;
use scraper::{selectable::Selectable, Html};
use serde::{Deserialize, Serialize};
use serde_json::json;
use futures::{stream, StreamExt, TryStreamExt};
use crate::{client::{AjaxClient, AjaxResponse}, error::{AjaxClientError, ParseElementError, TargetNotExist, WikidotError}, page::Page, parser, selectors};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Site{
    pub client: AjaxClient,
    pub id: i32,
    pub title: String,
    pub unix_name: String,
    pub ssl_supported: bool,
}

impl Site{
    pub fn url(&self) -> String{
        let name = &self.unix_name;
        let s = if self.ssl_supported {"s"} else {""};
        format!("http{s}://{name}.wikidot.com")
    }

    pub async fn request(&self, param: &[(&str, &str)]) -> Result<AjaxResponse, AjaxClientError>{
        let url = &self.url();
        self.client.request(param, &format!("{url}/ajax-module-connector.php")).await
    }

    pub async fn search(&self, param: &[(&str, &str)]) -> Result<Vec<Page>, WikidotError>{
        let properties = [
            "fullname",
            "category",
            "name",
            "title",
            "created_at",
            "created_by_linked",
            "updated_at",
            "updated_by_linked",
            "commented_at",
            "commented_by_linked",
            "parent_fullname",
            "comments",
            "size",
            "children",
            "rating_votes",
            "rating",
            "rating_percent",
            "revisions", 
            "tags",
            "_tags",
        ];
        let mut module_body = String::from("[[div class=\"page\"]]\n");
        properties.map(|property|
                module_body.push_str(&format!(
                        r#"[[span class="set {property}"]]
                        [[span class="name"]] {property} [[/span]]
                        [[span class="value"]] %%{property}%% [[/span]]
                        [[/span]]"#
                )
            )
        );
        module_body.push_str("\n[[/div]]");
        let mut param_vec = Vec::from([
            ("moduleName", "list/ListPagesModule"),
            ("module_body", &module_body),
            ("perPage", "250")
        ]);
        param_vec.extend_from_slice(param);
        let result = self.request(param_vec.as_slice()).await?;
        let fragment = Html::parse_fragment(&result.body);
        let no_re = Regex::new(r"of (\d+)")?;
        
        let page_num = match fragment.select(&selectors::PAGERNO).next(){
        Some(val) => no_re.captures(&val.text().collect::<String>()).ok_or(ParseElementError::page_num())?
            .get(1).ok_or(ParseElementError::page_num())?
            .as_str().parse::<i16>()?,
        None => 1,
        };

        let mut tasks = Vec::new();
        let module_arc = Arc::new(&module_body);

        for i in 2..page_num{
            let module_arc_clone = module_arc.clone();
            let num = (i * 250).to_string();
            tasks.push(async move{
                let mut single_vec = Vec::from(&[
                        ("moduleName", "list/ListPagesModule"),
                        ("module_body", &module_arc_clone),
                        ("perPage", "250"),
                        ("offset", &num)
                    ]);
                single_vec.extend_from_slice(&param);
                self.request(&single_vec).await
            })
        }

        let mut results = stream::iter(tasks).buffer_unordered(self.client.config.semaphore_limit as usize)
            .try_collect::<Vec<_>>().await?
            .into_iter().map(|x| Html::parse_fragment(&x.body))
            .collect::<Vec<_>>();
        results.extend_from_slice(&[fragment]);

        let mut pages_vec = Vec::new();

        for body in results{
            for page in body.select(&selectors::PAGE){
                let mut page_properties = json!({});
                let page_object = page_properties.as_object_mut().ok_or(ParseElementError::site_ele())?;
                let mut tags = Vec::new();
                for set_ele in page.select(&selectors::SET){
                    let key = set_ele.select(&selectors::KEY).next().ok_or(ParseElementError::site_ele())?.text().collect::<String>();

                    let value = if let Some(ele) = set_ele.select(&selectors::VALUE).next() {
                        let val_str = ele.text().collect::<String>().trim().to_string();

                        if ["tags", "_tags"].contains(&key.as_str()){
                            for tag in val_str.split(" "){
                                tags.push(tag.to_string())
                            }
                            continue;
                        }
                        else if ["created_at", "updated_at", "commented_at"].contains(&key.as_str()){
                            json!(parser::odate(ele))
                        }
                        else if  [
                            "created_by_linked",
                            "updated_by_linked",
                            "commented_by_linked",
                        ].contains(&key.as_str()){
                            json!(parser::printuser(ele)?)
                        }
                        else if ["rating_votes", "comments", "size", "revisions", "children"].contains(&key.as_str()){
                            json!(val_str.parse::<i32>()?)
                        }
                        else if ["rating"].contains(&key.as_str()){
                            json!(val_str.parse::<f64>()?)
                        }
                        else if ["rating_percent"].contains(&key.as_str()){
                            if page.select(&selectors::STAR).next().is_some() {json!(val_str.parse::<f64>()? / 100.0)}
                            else {json!(None::<Option<f64>>)}
                        }
                        else {json!(val_str)}
                    }
                    else {json!(None::<Option<String>>)};

                    let key = 
                        if key.contains("_linked") {key.replace("_linked", "")}
                        else if ["comments", "children", "revisions"].contains(&key.as_str()) {format!("{}_count", key)}
                        else if &key == "rating_votes" {"votes_count".to_string()}
                        else {key};

                    page_object.insert(key, value);
                }

                page_object.insert("tags".to_string(), json!(tags));
                page_object.insert("site".to_string(), json!(self.clone()));
                pages_vec.push(serde_json::from_value::<Page>(page_properties)?)
            }
        }

        Ok(pages_vec)
    }
}

impl AjaxClient {
    pub async fn get_site(&self, name: &str) -> Result<Site, WikidotError>{
        let _response = self.get(&format!("https://{name}.wikidot.com")).await?;
        let (_response, ssl_supported) = match _response.status(){
            StatusCode::NOT_FOUND => return Err(TargetNotExist::site())?,
            StatusCode::MOVED_PERMANENTLY => (self.get(&format!("http://{name}.wikidot.com")).await?, false),
            _ => (_response, true)
        };
        let _text = _response.text().await?;

        let id_re = Regex::new(r"WIKIREQUEST\.info\.siteId = (\d+);")?;
        let id = id_re.captures(&_text).ok_or(ParseElementError::site_id())?.get(1).ok_or(ParseElementError::site_id())?.as_str().parse::<i32>()?;

        let title_re = Regex::new(r"<title>(.*?)</title>")?;
        let title = title_re.captures(&_text).ok_or(ParseElementError::site_title())?.get(1).ok_or(ParseElementError::site_title())?.as_str();

        Ok(Site{
            client: self.clone(),
            id,
            title: title.to_string(),
            unix_name: name.to_string(),
            ssl_supported
        })
    }
}