use std::path::Path;

use rusqlite::{params, Result};
use tokio_rusqlite::Connection;

use crate::{scrapers::scraper_2022, insights::insights_2022};

static mut conn: Option<Connection> = None;

pub(crate) async fn initialize(path: &Path){
    unsafe{
        conn = Some(Connection::open(path).await.unwrap());
    }
}

pub(crate) async fn scrapeAndUpdate(){
    unsafe{
        let localconn = ((&conn).as_ref()).unwrap();
        scraper_2022::scrape(&localconn).await;

        println!("Finished Scraping Year 2022");

        insights_2022::initialize(Path::new("cache/2022/cache.db")).await;
        insights_2022::updateCache(localconn).await;
    }
}