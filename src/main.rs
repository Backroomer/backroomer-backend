use std::{collections::HashMap, thread::sleep, time::Duration};
use futures::{stream, StreamExt};
use mongodb::bson::{doc, DateTime};
use scraper::{ElementRef, Html};
use dotenv;
use wikidot::{client::AjaxClient, error::{ParseElementError, WikidotError}, mongo_page::{update_alt_titles, update_page, MongoPage}, mongo_user::{add_user, update_user, MongoUser, USER_ADD, USER_NOW}, page::PAGE_VEC, parser, selectors, site::Site};

const ALT_TITLE_URLS: [&str; 12] = [
    "https://backrooms-wiki-cn.wikidot.com/normal-levels-i",
    "https://backrooms-wiki-cn.wikidot.com/sub-layers",
    "https://backrooms-wiki-cn.wikidot.com/enigmatic-levels",
    "https://backrooms-wiki-cn.wikidot.com/objects",
    "https://backrooms-wiki-cn.wikidot.com/phenomena",
    "https://backrooms-wiki-cn.wikidot.com/normal-levels-cn-i",
    "https://backrooms-wiki-cn.wikidot.com/normal-levels-cn-ii",
    "https://backrooms-wiki-cn.wikidot.com/sub-layers-cn",
    "https://backrooms-wiki-cn.wikidot.com/enigmatic-series-cn",
    "https://backrooms-wiki-cn.wikidot.com/entities-cn",
    "https://backrooms-wiki-cn.wikidot.com/objects-cn",
    "https://backrooms-wiki-cn.wikidot.com/phenomena-cn",
];

macro_rules! collect_result {
    ($hash: expr, $results: expr, $iter: expr) => {
        for result in $results{
            let item = $iter.next().unwrap();
            if result.is_err() {
                $hash.insert(item, result.err().unwrap());
            }
        }
    };
}

async fn acquire_metadata(
    tr: ElementRef<'_>, 
    site: Site,
    page_col: mongodb::Collection<MongoPage>
) -> Result<(), WikidotError> {
    let mut tds = tr.select(&selectors::TD);
    let page_fullname = tds.next().ok_or(ParseElementError::page_ele())?.text().collect::<String>();
    let user_name = tds.next().ok_or(ParseElementError::user_ele())?.text().collect::<String>();
    if user_name.is_empty() {return  Ok(())}
    let user_res = site.request(&[
        ("threadId", "15081869"),
        ("source", format!("[[*user {}]]", &user_name).as_str()),
        ("moduleName", "forum/ForumPreviewPostModule"),
        ("parentId", ""),
        ("title", "")
    ]).await?;
    let user_html = Html::parse_fragment(&user_res.body);

    if let Some(user_ele) = user_html.select(&selectors::PRINTUSER).nth(1){
        match parser::printuser(user_ele)?.id {
            None => {return Ok(())},
            Some(8528464) => {return Ok(())},
            Some(data_id) => {
                println!("{}, {}", page_fullname, user_name);
                page_col.update_one(doc! {"fullname": page_fullname, "author": {"$ne": data_id}}, 
                doc! { "$push": { "author": data_id } }).await?;
            },
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenv::from_filename(".env.local")?;
    let mongo = mongodb::Client::with_uri_str(dotenv::var("DB_LINK")?)
        .await?;
    let db = mongo.database("backrooms-cn");
    let page_col: mongodb::Collection<MongoPage> = db.collection("pages");
    let user_col: mongodb::Collection<MongoUser> = db.collection("users");
    let client = AjaxClient::from(&dotenv::var("WD_USERNAME")?,
        &dotenv::var("WD_PASSWORD")?).await?;
    let semaphore = dotenv::var("SEMAPHORE")?.parse::<usize>()?;
    loop {
        let start = DateTime::now();
        println!("start: {:?}", start.timestamp_millis());

        USER_ADD.lock()?.clear();
        USER_NOW.lock()?.clear();
        PAGE_VEC.lock()?.clear();

        for user_bson in user_col.distinct("id", doc! {}).await?{
            USER_NOW.lock()?.push(user_bson.as_i32().unwrap());
        }
        let site = client.get_site("backrooms-wiki-cn").await?;

        let pages = site.search(&[("category", "*")]).await?;
        
        let results = stream::iter(
            pages.iter()
                .map(|page| update_page(page_col.clone(), page.clone()))
        )
        .buffered(semaphore)
        .collect::<Vec<_>>()
        .await;

        let mut page_hash: HashMap<_, _> = HashMap::new();
        let mut user_hash: HashMap<_, _> = HashMap::new();

        for (i, result) in results.into_iter().enumerate() {
            if result.is_err() {
                page_hash.insert(pages[i].fullname.clone(), result.err().unwrap());
            }
        }

        let _ = page_col.update_many(doc! { "id": { "$nin": PAGE_VEC.lock()?.clone() } }, doc! { "$set": {"status": false}}).await?;
        
        let response = client.get("https://backrooms-wiki-cn.wikidot.com/attribution-metadata").await?.text().await?;
        let html = Html::parse_document(&response);
        let table = html.select(&selectors::TABLE).next().unwrap();
        let _ = stream::iter(
            table.select(&selectors::TR)
                .skip(1)
                .map(|tr| acquire_metadata(tr, site.clone(), page_col.clone()))
        )
        .buffered(semaphore)
        .collect::<Vec<_>>()
        .await;
        
        println!("{:?}, {:?}, {:?}", 
            PAGE_VEC.lock()?, 
            USER_NOW.lock()?, 
            USER_ADD.lock()?
        );

        let update_users: Vec<i32> = user_col
            .distinct("id", doc! {"account_type": {"$ne": "deleted"}}).await?
            .into_iter()
            .map(|bson| bson.as_i32().unwrap())
            .collect();

        let results = stream::iter(
            update_users.iter()
                .map(|&user_id| update_user(user_col.clone(), user_id))
        )
        .buffered(semaphore)
        .collect::<Vec<_>>()
        .await;
        collect_result!(user_hash, results, update_users.iter().copied());

        let results = stream::iter(
            USER_ADD.lock()?.to_vec()
                .iter()
                .map(|user_id| add_user(user_col.clone(), *user_id))
        )
        .buffered(semaphore)
        .collect::<Vec<_>>()
        .await;
        collect_result!(user_hash, results, USER_ADD.lock()?.to_vec().iter().copied());

        println!("failed pages: {:?}", page_hash);
        println!("failed users: {:?}", user_hash);

        for url in ALT_TITLE_URLS{
            update_alt_titles(client.clone(), url, page_col.clone()).await?;
        }

        let end = DateTime::now();
        println!("end: {}, duration: {}", end.timestamp_millis(), end.saturating_duration_since(start).as_secs());

        sleep(Duration::from_secs(21600));
    }
}