use std::{collections::HashMap, sync::Arc, thread::sleep, time::Duration};
use futures::{stream, StreamExt};
use mongodb::bson::{doc, DateTime};
use scraper::{ElementRef, Html, Selector};
use dotenv;
use wikidot::{client::AjaxClient, error::WikidotError, mongo_page::{update_page, MongoPage}, mongo_user::{add_user, update_user, MongoUser, USER_ADD, USER_NOW}, page::PAGE_VEC, parser, selectors, site::Site};

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

async fn acquire_metadata(tr: ElementRef<'_>, site: Arc<Site>, page_col: Arc<mongodb::Collection<MongoPage>>) -> Result<(), WikidotError>{
    let mut tds = tr.select(&selectors::TD);
    let page_fullname = tds.next().unwrap().text().collect::<String>();
    let user_name = tds.next().unwrap().text().collect::<String>();
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
    dotenv::from_filename(".env.local").unwrap();
    let mongo = mongodb::Client::with_uri_str(dotenv::var("DB_LINK").unwrap())
        .await?;
    let db = mongo.database("backrooms-cn");
    let page_col: Arc<mongodb::Collection<MongoPage>> = Arc::new(db.collection("pages"));
    let user_col: Arc<mongodb::Collection<MongoUser>> = Arc::new(db.collection("users"));
    loop {
        let start = DateTime::now();
        println!("start: {:?}", start.timestamp_millis());
        unsafe {
            USER_ADD = Vec::new();
            USER_NOW = Vec::new();
            PAGE_VEC = Vec::new();
        }
        for user_bson in user_col.distinct("id", doc! {}).await?{
            unsafe {USER_NOW.push(user_bson.as_i32().unwrap());}
        }
        let client = AjaxClient::from(&dotenv::var("WD_USERNAME").unwrap(),
            &dotenv::var("WD_PASSWORD").unwrap()).await?;
        let site = client.get_site("backrooms-wiki-cn").await?;

        let pages = site.search(&[("category", "*")]).await?;
        let mut page_iter = pages.clone().into_iter();

        let mut tasks = Vec::new();
        
        for page in pages{
            let col_arc_clone = page_col.clone();
            tasks.push(update_page(col_arc_clone, page));
        }

        let results = stream::iter(tasks).buffered(6)
            .collect::<Vec<_>>().await;

        let mut page_hash: HashMap<_, _> = HashMap::new();
        let mut user_hash: HashMap<_, _> = HashMap::new();

        for result in results{
            let page = page_iter.next().unwrap();
            if result.is_err() {
                page_hash.insert(page.fullname, result.err().unwrap());
            }
        }

        let _ = page_col.update_many(doc! { "id": { "$nin": unsafe { PAGE_VEC.clone() } } }, doc! { "$set": {"status": false}}).await?;
        
        let response = client.get("https://backrooms-wiki-cn.wikidot.com/attribution-metadata").await?.text().await?;
        let html = Html::parse_document(&response);
        let table = html.select(&Selector::parse("table.wiki-content-table").unwrap()).next().unwrap();
        let site_arc = Arc::new(site);
        let mut tasks = Vec::new();
        for tr in table.select(&selectors::TR).skip(1) {
            let col_arc_clone = page_col.clone();
            let site_clone = site_arc.clone();
            tasks.push(acquire_metadata(tr, site_clone, col_arc_clone));
        }

        let _ = stream::iter(tasks).buffered(6)
            .collect::<Vec<_>>().await;
        
        unsafe { println!("{:?}, {:?}, {:?}", PAGE_VEC, USER_NOW, USER_ADD) };

        let mut tasks = Vec::new();
        unsafe{
            for user_id in USER_NOW.clone() {
                let col_arc_clone = user_col.clone();
                tasks.push(update_user(col_arc_clone, user_id));
            }
        }

        let results = stream::iter(tasks).buffered(6)
            .collect::<Vec<_>>().await;
        collect_result!(user_hash, results, unsafe { USER_NOW.clone().into_iter() });

        let mut tasks = Vec::new();
        unsafe{
            for user_id in USER_ADD.clone() {
                let col_arc_clone = user_col.clone();
                tasks.push(add_user(col_arc_clone, user_id));
            }
        }

        let results = stream::iter(tasks).buffered(6)
            .collect::<Vec<_>>().await;
        collect_result!(user_hash, results, unsafe { USER_ADD.clone().into_iter() });

        println!("failed pages: {:?}", page_hash);
        println!("failed users: {:?}", user_hash);
        let end = DateTime::now();
        println!("end: {}, duration: {}", end.timestamp_millis(), end.saturating_duration_since(start).as_secs());

        sleep(Duration::from_secs(21600));
    }
}