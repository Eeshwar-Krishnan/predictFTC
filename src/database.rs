use std::{path::Path, collections::HashMap};

use rusqlite::{params, Result};
use tokio_rusqlite::Connection;

use crate::{scrapers::scraper_2022::{self, Match_2022, Match_2022_Results}, insights::insights_2022};

static mut conn: Option<Connection> = None;

pub struct Event{
    eventcode: String,
    eventname: String,
    timestart: i32
}

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

        println!("Finished Updating Cache");
    }
}

pub(crate) async fn scrapeAndUpdateToday(){
    unsafe{
        scraper_2022::scrapeToday((&conn).as_ref().unwrap()).await;
    }
}

pub(crate) async fn getTeamNameMap(year: i32) -> HashMap<i32, String>{
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

//SELECT * FROM 'teams2022' WHERE teamNum LIKE 'foo%' OR teamName LIKE '%foo%'
pub(crate) async fn getTeams(year: i32, partial: Option<String>) -> HashMap<i32, String>{
    let yearlocal = year.clone();
    unsafe{
        let localconn = ((&conn).as_ref()).unwrap();
        
        return localconn.call(move |conn2|{
            let mut map: HashMap<i32, String> = HashMap::new();
            match(partial){
                Some(val) => {
                    conn2.prepare(&format!("SELECT teamNum, teamName FROM teams{} WHERE teamNum LIKE :partial OR teamName LIKE :partial2 LIMIT 0,50", yearlocal)).unwrap().query_map(&[(":partial", format!("{}%", val).to_string().as_str()), (":partial2", format!("%{}%", val).to_string().as_str())], |row|{
                        let teamNum: String = row.get(0).unwrap();
                        let teamName: String = row.get(1).unwrap();
                        map.insert(teamNum.parse::<i32>().unwrap(), teamName);
                        return (Ok(1));
                    }).unwrap().collect::<Result<Vec<i32>, rusqlite::Error>>().unwrap();
                    return map;
                },
                None => {
                    conn2.prepare(&format!("SELECT teamNum, teamName FROM teams{}", yearlocal)).unwrap().query_map([], |row|{
                        let teamNum: String = row.get(0).unwrap();
                        let teamName: String = row.get(1).unwrap();
                        map.insert(teamNum.parse::<i32>().unwrap(), teamName);
                        return (Ok(1));
                    }).unwrap().collect::<Result<Vec<i32>, rusqlite::Error>>().unwrap();
                    return map;
                }
            }
        }).await;
    }
}

pub(crate) async fn getDistricts(year: i32) -> HashMap<i32, String>{
    let yearlocal = year.clone();
    unsafe{
        let localconn = ((&conn).as_ref()).unwrap();
        
        return localconn.call(move |conn2|{
            let mut map: HashMap<i32, String> = HashMap::new();
            conn2.prepare(&format!("SELECT teamNum, region FROM teams{}", yearlocal)).unwrap().query_map([], |row|{
                let teamNum: String = row.get(0).unwrap();
                let teamName: String = row.get(1).unwrap();
                map.insert(teamNum.parse::<i32>().unwrap(), teamName);
                return (Ok(1));
            }).unwrap().collect::<Result<Vec<i32>, rusqlite::Error>>().unwrap();
            return map;
        }).await;
    }
}

pub(crate) async fn getNumInDistrict(year: i32, district: String) -> i32{
    let yearlocal = year.clone();
    unsafe{
        let localconn = ((&conn).as_ref()).unwrap();
        
        return localconn.call(move |conn2|{
            let mut map: HashMap<i32, String> = HashMap::new();
            let mut query = conn2.prepare(&format!("SELECT COUNT(*) FROM teams{} WHERE region=:region", yearlocal)).unwrap();
            let mut rows = query.query(&[(":region", district.to_string().as_str())]).unwrap();

            let count: i32 = rows.next().unwrap().unwrap().get(0).unwrap();
            return count;
        }).await;
    }
}

pub(crate) async fn getEvents(year: i32) -> Vec<Event>{
    let yearlocal = year.clone();
    unsafe{
        let localconn = ((&conn).as_ref()).unwrap();
        
        return localconn.call(move |conn2|{
            let mut vec: Vec<Event> = Vec::new();
            conn2.prepare(&format!("SELECT eventcode, eventname, timestart FROM events{}", yearlocal)).unwrap().query_map([], |row|{
                let eventcode: String = row.get(0).unwrap();
                let eventname: String = row.get(1).unwrap();
                let timestart: String = row.get(2).unwrap();
                vec.push(Event{ eventcode, eventname, timestart: timestart.parse::<i32>().unwrap() });
                return (Ok(1));
            }).unwrap().collect::<Result<Vec<i32>, rusqlite::Error>>().unwrap();
            return vec;
        }).await;
    }
}

pub(crate) async fn getMatches2022(eventcode: Option<String>, team: Option<String>) -> Vec<Match_2022>{
    unsafe{
        return scraper_2022::deserialize((&conn).as_ref().unwrap(), eventcode, team).await;
    }
}

pub(crate) async fn getMatches2022Response(eventcode: Option<String>) -> Match_2022_Results{
    unsafe{
        return Match_2022_Results{
            matches: scraper_2022::deserialize((&conn).as_ref().unwrap(), eventcode, None).await
        }
    }
}