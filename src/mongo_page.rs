use std::{collections::HashMap, sync::Arc};
use mongodb::bson::{doc, DateTime};
use serde::{Deserialize, Serialize};
use crate::{error::WikidotError, page::Page, page_history::Revision};

#[derive(Deserialize, Serialize)]
pub struct MongoRateHistory{
    timestamp: DateTime,
    votes: HashMap<String, i8>,
    up: i16,
    down: i16,
}

#[derive(Deserialize, Serialize)]
pub struct MongoRevision{
    index: i16,
    id: i32,
    types: Vec<char>,
    created_by: i32,
    created_at: Option<DateTime>,
    comment: String,
}

#[derive(Deserialize, Serialize)]
pub struct MongoPage{
    id: i32,
    author: Vec<i32>,
    fullname: String,
    title: String,
    tags: Vec<String>,
    source: String,
    rate_history: Vec<MongoRateHistory>,
    history: Vec<MongoRevision>,
    comments_count: i16,
    status: bool,
}

fn process_revisions(revisions: Vec<Revision>) -> Vec<MongoRevision>{
    let mut rev = Vec::new();
    for revision in revisions{
        rev.push(MongoRevision{
            index: revision.index,
            id: revision.id,
            types: revision.types,
            created_by: revision.created_by.id.unwrap(),
            created_at: revision.created_at,
            comment: revision.comment,
        });
    }
    rev
}

pub async fn update_page(collection: Arc<mongodb::Collection<MongoPage>>, mut page: Page) -> Result<(), WikidotError>{
    println!("{}", page.fullname);
    let mut new_rates: HashMap<String, i8> = HashMap::new();
    let mongo_page;
    let up: i16 = (page.votes_count + page.rating as i16) / 2;
    let down: i16 = (page.votes_count - page.rating as i16) / 2;
    let filter = if let Some(user_id) = page.created_by.id {
        doc! {
            "$expr": {"$eq": [{"$arrayElemAt": ["$history.created_at", -1]}, page.created_at]},
            "author": user_id,
        }
    }
    else {
        doc! {"$expr": {"$eq": [{"$arrayElemAt": ["$history.created_at", -1]}, page.created_at]}}
    };
    if let Some(mut old_page) = collection.find_one(filter).await? {
        page.id = Some(old_page.id);
        if page.updated_at != old_page.rate_history[0].timestamp{
            old_page.source = page.acquire_page_source().await?;
            old_page.history = process_revisions(page.acquire_revisions(&["all"]).await?);
        }

        let mut old_rates = old_page.rate_history;
        let last = old_rates.pop().unwrap();

        let mut diff: HashMap<String, i8> = HashMap::new();
        for vote in page.acquire_votes().await?{
            let user_id = vote.user.id.unwrap();
            if let Some(matched_vote) = last.votes.get(&user_id.to_string()){
                if matched_vote != &vote.rate{
                    diff.insert(vote.user.id.unwrap().to_string(), *matched_vote);
                }
            }
            else{
                diff.insert(user_id.to_string(), 0);
            }
            new_rates.insert(user_id.to_string(), vote.rate);
        }
        old_rates.push(MongoRateHistory{votes: diff, ..last});
        old_rates.push(MongoRateHistory{timestamp: DateTime::now(), votes: new_rates, up, down});
        
        mongo_page = MongoPage{
            fullname: page.fullname,
            author: vec![page.created_by.id.unwrap_or(old_page.history.last().unwrap().created_by)],
            title: page.title.unwrap_or(String::new()),
            tags: page.tags,
            rate_history: old_rates,
            comments_count: page.comments_count,
            status: true,
            ..old_page
        };
        collection.replace_one(doc! {"id": old_page.id}, &mongo_page).await?;
    }
    else {
        page.acquire_id().await?;
        for vote in page.acquire_votes().await?{
            new_rates.insert(vote.user.id.unwrap().to_string(), vote.rate);
        }
        let revisions = page.acquire_revisions(&["all"]).await?;
        let source = page.acquire_page_source().await?;

        mongo_page = MongoPage{
            id: page.id.unwrap(),
            author: vec![page.created_by.id.unwrap_or(revisions.last().unwrap().created_by.id.unwrap())],
            fullname: page.fullname,
            title: page.title.unwrap_or(String::new()),
            source,
            tags: page.tags,
            rate_history: vec![MongoRateHistory{timestamp: DateTime::now(), votes: new_rates, up, down}],
            history: process_revisions(revisions),
            comments_count: page.comments_count,
            status: true,
        };
        collection.insert_one(&mongo_page).await?;
    }

    Ok(())
}