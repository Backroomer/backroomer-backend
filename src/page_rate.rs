use scraper::Html;
use serde::{Deserialize, Serialize};
use crate::{error::WikidotError, page::Page, parser, selectors, user::User};

#[derive(Serialize, Deserialize)]
pub struct RateUser{
    pub user: User,
    pub rate: i8,
}

impl Page {
    pub async fn acquire_votes(&mut self) -> Result<Vec<RateUser>, WikidotError>{
        let page_id = self.acquire_id().await?;

        let response = self.site.request(&[
            ("pageId", &page_id.to_string()),
            ("moduleName", "pagerate/WhoRatedPageModule")
            ]).await?;
        
        let body = Html::parse_fragment(&response.body);

        let mut rate_vec = Vec::new();

        for (user_ele, vote_ele) in body.select(&selectors::PRINTUSER).into_iter().zip(body.select(&selectors::VOTE)){
            rate_vec.push(RateUser{
                user: parser::printuser(user_ele)?,
                rate: match vote_ele.text().collect::<String>(){
                    val if val.contains('+') => 1,
                    val if val.contains('-') => -1,
                    _ => 0,
                },
            });
        }

        Ok(rate_vec)
    }
}