use mongodb::bson::DateTime;
use scraper::Html;
use serde::{Deserialize, Serialize};
use crate::{client::AjaxClient, error::{ParseElementError, WikidotError}, parser, selectors, site::Site};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct User{
    pub id: Option<i32>,
    pub name: String,
    pub unix_name: Option<String>,
    pub ip: Option<String>,
    pub user_type: String,
}

#[derive(Debug)]
pub struct UserProperty{
    pub title: String,
    pub since: DateTime,
    pub avatar: String,
    pub account_type: String,
    pub karma: i8,
}

impl Default for User {
    fn default() -> Self {
        User{
            id: None,
            name: "account deleted".to_string(),
            unix_name: None,
            ip: None,
            user_type: "DeletedUser".to_string(),
        }
    }
}

impl User{
    pub fn from_wikidot_user() -> Self{
        User{
            name: "Wikidot".to_string(),
            unix_name: Some("wikidot".to_string()),
            user_type: "WikidotUser".to_string(),
            ..User::default()
        }
    }

    pub fn from_guest_user(name: String) -> Self{
        User{
            name,
            user_type: "GuestUser".to_string(),
            ..User::default()
        }
    }

    pub fn from_deleted_user(id: Option<i32>) -> Self{
        User{
            id,
            unix_name: Some("account_deleted".to_string()),
            ..User::default()
        }
    }

    pub fn from(id: i32, name: String, unix_name: String) -> Self{
        User{
            id: Some(id),
            name,
            unix_name: Some(unix_name),
            ip: None,
            user_type: "NormalUser".to_string(),
        }
    }

    pub fn avatar(&self) -> Option<String>{
        Some(format!("http://www.wikidot.com/avatar.php?userid={}", self.id?))
    }
}

impl Site{
    pub async fn member_of_site_since(&self, user_id: i32) -> Option<DateTime>{
        parser::odate(
            Html::parse_fragment(
            &self.request(&[
                ("user_id", &user_id.to_string()),
                ("moduleName", "users/UserInfoWinModule"),
            ]).await.ok()?.body
        ).select(&selectors::ODATE)
        .nth(1)?)
    }
}

impl AjaxClient{
    pub async fn user(&self, user_id: i32) -> Result<UserProperty, WikidotError>{
        let html = Html::parse_fragment(
            &self.request(&[
                ("user_id", &user_id.to_string()),
                ("moduleName", "users/UserInfoWinModule"),
            ], "https://www.wikidot.com/ajax-module-connector.php")
            .await?.body);

        let since = parser::odate(html.root_element()).ok_or(ParseElementError::user_date())?;
        let title = html.select(&selectors::H1).next()
            .ok_or(ParseElementError::user_ele())?.text().collect::<String>();
        let avatar_url = html.select(&selectors::IMG).next()
            .ok_or(ParseElementError::user_ele())?.attr("src")
            .ok_or(ParseElementError::user_avatar())?;
        let avatar = self.get(avatar_url).await?.headers().get("location")
            .ok_or(ParseElementError::user_avatar())?.to_str()?.to_string();
        let mut account_type = "free".to_string();
        let mut karma: i8 = 0;
        for tr in html.select(&selectors::TR){
            let mut tds = tr.select(&selectors::TD)
                .map(|x| x.text().collect::<String>().trim().to_string());

            match tds.next().ok_or(ParseElementError::user_ele())?.as_str(){
                "Account type" => account_type = if title.is_empty() {"deleted".to_string()} 
                    else {tds.next().ok_or(ParseElementError::user_ele())?},
                "Karma level" => karma = match tds.next().ok_or(ParseElementError::user_ele())?[..11].trim() {
                    "low" => 1,
                    "medium" => 2,
                    "high" => 3,
                    "very high" => 4,
                    "guru" => 5,
                    _ => 0,
                },
                _ => (),
            }
        }

        Ok(UserProperty{
            title,
            since,
            avatar,
            account_type,
            karma,
        })
    }
}