use mongodb::bson::{doc, DateTime};
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::{client::AjaxClient, error::{ParseElementError, WikidotError}};

pub static USER_ADD: Lazy<Mutex<Vec<i32>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static USER_NOW: Lazy<Mutex<Vec<i32>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Deserialize, Serialize)]
pub struct MongoUser{
    pub id: i32,
    join: DateTime,
    title: Vec<String>,
    avatar: Vec<MongoAvatar>,
    karma: Vec<MongoKarma>,
    account_type: String,
}

#[derive(Deserialize, Serialize)]
struct MongoKarma{
    level: i8,
    timestamp: DateTime,
}

#[derive(Deserialize, Serialize)]
struct MongoAvatar{
    image: String,
    timestamp: DateTime,
}

pub async fn update_user(collection: mongodb::Collection<MongoUser>, user_id: i32) -> Result<(), WikidotError>{
    let user = AjaxClient::new().user(user_id).await?;
    println!("{:?}", user.title);
    let mut user_history = match collection.find_one(doc! {"id": user_id}).await? {
        Some(history) => history,
        None => return Ok(()),
    };
    if user_history.title.last().ok_or(ParseElementError::mongo_ele())? != &user.title && !user.title.is_empty(){
        user_history.title.push(user.title);
    }

    if user_history.avatar.last().ok_or(ParseElementError::mongo_ele())?.image != user.avatar{
        user_history.avatar.push(MongoAvatar{image: user.avatar, timestamp: DateTime::now()});
    }

    if user_history.karma.last().ok_or(ParseElementError::mongo_ele())?.level < user.karma{
        user_history.karma.push(MongoKarma{level: user.karma, timestamp: DateTime::now()});
    }

    let _ = collection.replace_one(doc! {"id": user_id}, MongoUser{
        id: user_id,
        join: user.since, 
        account_type: user.account_type, 
        ..user_history
    }).await?;

    Ok(())
}

pub async fn add_user(collection: mongodb::Collection<MongoUser>, user_id: i32) -> Result<(), WikidotError>{
    let user = AjaxClient::new().user(user_id).await?;
    println!("{:?}", user.title);
    let _ = collection.insert_one(MongoUser{
        id: user_id,
        join: user.since, 
        account_type: user.account_type, 
        title: vec![user.title],
        avatar: vec![MongoAvatar{image: user.avatar, timestamp: DateTime::now()}],
        karma: vec![MongoKarma{level: user.karma, timestamp: DateTime::now()}],
    }).await?;

    Ok(())
}