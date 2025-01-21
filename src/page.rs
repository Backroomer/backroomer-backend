use mongodb::bson::DateTime;
use regex::Regex;
use reqwest::StatusCode;
use scraper::Html;
use serde::{Deserialize, Serialize};
use crate::{error::{ParseElementError, IdNotFound, TargetNotExist, WikidotError}, selectors, site::Site, user::User};

pub static mut PAGE_VEC: Vec<i32> = Vec::new();

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Page{
    pub id: Option<i32>,
    pub site: Site,
    pub fullname: String,
    pub name: String,
    pub category: String,
    pub title: Option<String>,
    pub children_count: i16,
    pub comments_count: i16,
    pub size: i32,
    pub rating: f64,
    pub votes_count: i16,
    pub rating_percent: Option<f64>,
    pub revisions_count: i16,
    pub parent_fullname: Option<String>,
    pub tags: Vec<String>,
    pub created_by: User,
    pub created_at: DateTime,
    pub updated_by: User,
    pub updated_at: DateTime,
    pub commented_by: Option<User>,
    pub commented_at: Option<DateTime>,
}

impl Page{
    pub async fn acquire_page_source(&mut self) -> Result<String, WikidotError>{
        let page_id = self.acquire_id().await?;

        let response = self.site.request(&[
            ("page_id", &page_id.to_string()),
            ("moduleName", "viewsource/ViewSourceModule"),
        ]).await?;
        
        let body = Html::parse_fragment(&response.body);

        Ok(
            body.select(&selectors::PAGESOURCE).next().ok_or(ParseElementError::revision_ele())?
            .text().collect::<String>()
            .trim().to_string()
        )
    }
    
    pub async fn acquire_id(&mut self) -> Result<i32, WikidotError>{
        if let Some(id) = self.id {
            return Ok(id)
        }

        let _response = self.site.client.get(format!("{}/{}/norender/true", self.site.url(), self.fullname).as_str()).await?;

        if _response.status() == StatusCode::NOT_FOUND{
            return Err(TargetNotExist::page())?
        }

        let _text = _response.text().await?.to_string();

        let id_re = Regex::new(r"WIKIREQUEST\.info\.pageId = (\d+);")?;
        let id = id_re.captures(&_text).ok_or(IdNotFound::page())?.get(1).ok_or(IdNotFound::page())?.as_str().parse::<i32>()?;
        
        self.id = Some(id);
        unsafe { PAGE_VEC.push(id) };
        Ok(id)
    }
}