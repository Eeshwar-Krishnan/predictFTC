use std::{path::Path, collections::HashMap};

use rusqlite::{params, Result};
use tokio_rusqlite::Connection;

use crate::{scrapers::scraper_2022, insights::insights_2022};

static mut conn: Option<Connection> = None;

pub(crate) async fn initialize(path: &Path){
    unsafe{
        conn = Some(Connection::open(path).await.unwrap());
    }
    insights_2022::initialize(Path::new("cache/2022/cache.db")).await;
}

pub(crate) async fn scrapeAndUpdateAll(){
    println!("Scraping Matches!");
    unsafe{
        let localconn = ((&conn).as_ref()).unwrap();
        scraper_2022::scrape(&localconn, None, true).await;

        println!("Finished Scraping Year 2022");

        insights_2022::updateCache(localconn, None).await;
    }
}

pub(crate) async fn scrapeAndUpdateToday(){
    unsafe{
        scraper_2022::scrapeToday((&conn).as_ref().unwrap()).await;
    }
}

pub(crate) async fn getTeams(year: i32) -> HashMap<i32, String>{
    let yearlocal = year.clone();
    unsafe{
        let localconn = ((&conn).as_ref()).unwrap();
        
        return localconn.call(move |conn2|{
            let mut map: HashMap<i32, String> = HashMap::new();
            conn2.prepare(&format!("SELECT teamNum, teamName FROM teams{}", yearlocal)).unwrap().query_map([], |row|{
                let teamNum: String = row.get(0).unwrap();
                let teamName: String = row.get(1).unwrap();
                map.insert(teamNum.parse::<i32>().unwrap(), teamName);
                return (Ok(1));
            }).unwrap().collect::<Result<Vec<i32>, rusqlite::Error>>().unwrap();
            return map;
        }).await;
    }
}