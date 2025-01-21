use mongodb::bson::DateTime;
use regex::Regex;
use scraper::Html;
use serde::{Deserialize, Serialize};
use crate::{error::{ParseElementError, WikidotError}, page::Page, parser, selectors, user::User};

#[derive(Serialize, Deserialize, Debug)]
pub struct Revision{
    pub index: i16,
    pub id: i32,
    pub types: Vec<char>,
    pub created_by: User,
    pub created_at: Option<DateTime>,
    pub comment: String,
}

impl Page {
    pub async fn acquire_revisions(&mut self, options: &[&str]) -> Result<Vec<Revision>, WikidotError>{
        let page_id = self.acquire_id().await?;

        let mut option_str = String::from("{");
        for option in options{
            option_str.push_str(&format!("\"{option}\","));
        }
        option_str.push('}');
        let response = self.site.request(&[
            ("page", "1"),
            ("perpage", "10000"),
            ("page_id", &page_id.to_string()),
            ("options", &option_str),
            ("moduleName", "history/PageRevisionListModule"),
        ]).await?;

        let body = Html::parse_fragment(&response.body);

        let mut revision_vec = Vec::new();
        for revision in body.select(&selectors::TR).skip(1){
            let rev_id_re = Regex::new(r"\d+")?;
            let id = rev_id_re.captures(revision.attr("id").ok_or(ParseElementError::revision_id())?)
                .ok_or(ParseElementError::revision_id())?
                .get(0).ok_or(ParseElementError::revision_id())?
                .as_str()
                .parse::<i32>()?;
            let mut properties = revision.select(&selectors::TD);
                        
            let index = properties.next().ok_or(ParseElementError::revision_ele())?.text()
                .collect::<String>()
                .replace(".", "")
                .parse::<i16>()?;
            let types = properties.nth(1).ok_or(ParseElementError::revision_ele())?
                .text().collect::<String>()
                .chars().filter(|c| c.is_alphabetic())
                .collect::<Vec<char>>();
            let created_by = parser::printuser(properties.nth(1).ok_or(ParseElementError::revision_ele())?);
            let created_at = parser::odate(properties.next().ok_or(ParseElementError::revision_ele())?);
            let comment = properties.next().ok_or(ParseElementError::revision_ele())?.text().collect::<String>();

            revision_vec.push(Revision{
                index,
                id,
                types,
                created_by: created_by?,
                created_at,
                comment,
            });
        }

        Ok(revision_vec)
    }
}