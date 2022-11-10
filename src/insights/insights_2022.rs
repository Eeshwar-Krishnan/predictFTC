use std::{collections::HashMap, clone, path::Path};

use nalgebra::DMatrix;
use rusqlite::{Statement, Rows};
use serde::{Deserialize, Serialize};
use tokio_rusqlite::Connection;

use crate::database;

#[derive(Serialize, Deserialize)]
pub struct InsightTeam {
    teamNum: String,
    teamName: String,
    events: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct InsightTeamReturn{
    teams: Vec<InsightTeam>
}
#[derive(Serialize, Deserialize)]
pub struct EventCones{
    eventcode: String,
    cones: Vec<String>
}
#[derive(Serialize, Deserialize)]
pub struct EventConesReturn{
    events: Vec<EventCones>
}

static mut cacheConn: Option<Connection> = None;

pub(crate) async fn initialize(path: &Path){
    unsafe{
        cacheConn = Some(Connection::open(path).await.unwrap());
    }
}

pub(crate) async fn updateCache(connect: &Connection, eventslist: Option<Vec<String>>){
    let localCacheConn;
    unsafe{
        localCacheConn = (&cacheConn).as_ref().unwrap();
    }

    let events = match(eventslist){
        Some(val) => val,
        None => {
            connect.call(|conn|{
                return conn.prepare("SELECT DISTINCT eventcode FROM 'matches2022'").unwrap().query_map([], |row|{
                    return Ok(row.get(0).unwrap());
                }).unwrap().collect::<Result<Vec<String>, rusqlite::Error>>().unwrap();
            }).await
        },
    };

    localCacheConn.call(|conn|{
        conn.execute("CREATE TABLE IF NOT EXISTS cone_components (eventcode, team, dcJunctions11, dcJunctions12, dcJunctions13, dcJunctions14, dcJunctions15, dcJunctions21, dcJunctions22, dcJunctions23, dcJunctions24, dcJunctions25, dcJunctions31, dcJunctions32, dcJunctions33, dcJunctions34, dcJunctions35, dcJunctions41, dcJunctions42, dcJunctions43, dcJunctions44, dcJunctions45, dcJunctions51, dcJunctions52, dcJunctions53, dcJunctions54, dcJunctions55)", []).unwrap();
        conn.prepare("DELETE FROM cone_components").unwrap().execute([]).unwrap();
    }).await;

    for event in events{
        let map = match(computeEventConeComponents(event.clone(), connect).await){
            Some(val) => val,
            None => {
                continue;
            },
        };
        unsafe{
            localCacheConn.call(move |conn|{
                let localevent = event.clone();
                conn.execute("CREATE TABLE IF NOT EXISTS cone_components (eventcode, team, dcJunctions11, dcJunctions12, dcJunctions13, dcJunctions14, dcJunctions15, dcJunctions21, dcJunctions22, dcJunctions23, dcJunctions24, dcJunctions25, dcJunctions31, dcJunctions32, dcJunctions33, dcJunctions34, dcJunctions35, dcJunctions41, dcJunctions42, dcJunctions43, dcJunctions44, dcJunctions45, dcJunctions51, dcJunctions52, dcJunctions53, dcJunctions54, dcJunctions55)", []).unwrap();
                for teamnum in map.keys(){
                    let arr = map.get(teamnum).unwrap();
                    conn.execute("INSERT INTO cone_components (eventcode, team, dcJunctions11, dcJunctions12, dcJunctions13, dcJunctions14, dcJunctions15, dcJunctions21, dcJunctions22, dcJunctions23, dcJunctions24, dcJunctions25, dcJunctions31, dcJunctions32, dcJunctions33, dcJunctions34, dcJunctions35, dcJunctions41, dcJunctions42, dcJunctions43, dcJunctions44, dcJunctions45, dcJunctions51, dcJunctions52, dcJunctions53, dcJunctions54, dcJunctions55) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)",
                        [
                            localevent.clone(),
                            teamnum.to_string(),
                            arr.get(0).unwrap().to_string(),
                            arr.get(1).unwrap().to_string(),
                            arr.get(2).unwrap().to_string(),
                            arr.get(3).unwrap().to_string(),
                            arr.get(4).unwrap().to_string(),
                            arr.get(5).unwrap().to_string(),
                            arr.get(6).unwrap().to_string(),
                            arr.get(7).unwrap().to_string(),
                            arr.get(8).unwrap().to_string(),
                            arr.get(9).unwrap().to_string(),
                            arr.get(10).unwrap().to_string(),
                            arr.get(11).unwrap().to_string(),
                            arr.get(12).unwrap().to_string(),
                            arr.get(13).unwrap().to_string(),
                            arr.get(14).unwrap().to_string(),
                            arr.get(15).unwrap().to_string(),
                            arr.get(16).unwrap().to_string(),
                            arr.get(17).unwrap().to_string(),
                            arr.get(18).unwrap().to_string(),
                            arr.get(19).unwrap().to_string(),
                            arr.get(20).unwrap().to_string(),
                            arr.get(21).unwrap().to_string(),
                            arr.get(22).unwrap().to_string(),
                            arr.get(23).unwrap().to_string(),
                            arr.get(24).unwrap().to_string(),
                        ]).unwrap();
                }
            }).await;
        }
    }
}

async fn computeEventConeComponents(eventcode: String, connect: &Connection) -> Option<HashMap<i32, Vec<f32>>>{
    return connect.call(move |conn|{
        let mut numTeams: usize = conn.prepare(&format!("SELECT COUNT(*) FROM (SELECT team1 FROM 'matches2022' WHERE eventcode='{}' UNION SELECT team2 FROM 'matches2022' WHERE eventcode='{}')", eventcode.clone(), eventcode.clone())).unwrap().query([]).unwrap().next().unwrap().unwrap().get(0).unwrap();

        let mut stmt = conn.prepare(&format!("SELECT DISTINCT team1 FROM '{}' WHERE team1<30000 AND team2<30000 AND eventcode='{}' UNION SELECT DISTINCT team2 FROM '{}' WHERE team1<30000 AND team2<30000 AND eventcode='{}'", "matches2022", eventcode.clone(),  "matches2022", eventcode.clone())).unwrap();
        let mut rows = stmt.query([]).unwrap();

        let mut teammap: HashMap<isize, isize> = HashMap::new();
        let mut inverseteammap: HashMap<isize, isize> = HashMap::new();

        let mut index = 0;
        while let Some(row) = rows.next().unwrap(){
            teammap.insert(row.get(0).unwrap(), index);
            inverseteammap.insert(index, row.get(0).unwrap());
            index+=1;
        }

        let mut stmt = conn.prepare(&format!("SELECT team1,team2 FROM '{}' WHERE team1<30000 AND team2<30000 AND eventcode='{}'", "matches2022", eventcode.clone())).unwrap();
        let mut rows = stmt.query([]).unwrap();

        let mut teammatrix = DMatrix::from_element(numTeams, numTeams, 0.0);

        while let Some(row) = rows.next().unwrap(){
            let test: isize = row.get(1).unwrap();
            let team1: usize = *teammap.get(&row.get(0).unwrap()).unwrap() as usize;
            let team2: usize = *teammap.get(&row.get(1).unwrap()).unwrap() as usize;

            teammatrix[(team1, team1)] += 1.0;
            teammatrix[(team1, team2)] += 1.0;
            teammatrix[(team2, team1)] += 1.0;
            teammatrix[(team2, team2)] += 1.0;
        }

        let mut scorematrix = DMatrix::from_element(numTeams, 29, 0.0);
        let mut scoreRowSize = 0;

        for teamnum in teammap.keys(){
            let mut stmt2: Statement;
            stmt2 = conn.prepare(&format!("SELECT alliance, dcJunctions11, dcJunctions12, dcJunctions13, dcJunctions14, dcJunctions15, dcJunctions21, dcJunctions22, dcJunctions23, dcJunctions24, dcJunctions25, dcJunctions31, dcJunctions32, dcJunctions33, dcJunctions34, dcJunctions35, dcJunctions41, dcJunctions42, dcJunctions43, dcJunctions44, dcJunctions45, dcJunctions51, dcJunctions52, dcJunctions53, dcJunctions54, dcJunctions55 FROM matches2022 WHERE (team1={} OR team2={}) AND eventcode='{}'", teamnum, teamnum, eventcode.clone())).unwrap();
            let mut rows2 = stmt2.query([]).unwrap();
            
            let mut tmparray: [f64; 25] = [0.0; 25];
            while let Some(row) = rows2.next().unwrap(){
                for i in 0..5{
                    
                    for j in 0..5{
                        let alliance: String = row.get(0).unwrap();
                        let index = if(alliance == "Red") {1 + ((i * 5) + j)} else {1 + ((i * 5) + (4-j))};
                        let mut tmp: String = row.get(index).unwrap();
                        let mut count: f64 = 0.0;
                        
                        for c in tmp.chars(){
                            if(c == 'm'){
                                count += 1.0;
                            }
                        }
        
                        tmparray[((i * 5) + j)] += count;
                    }
                }
            }
            
            for i in 0..25{
                scorematrix[(*teammap.get(teamnum).unwrap() as usize, i)] = tmparray[i] as f64;
            }
        }
        
        let mut result = teammatrix.lu();
        let mut oprc = match(result.solve(&scorematrix)){
            Some(val) => val,
            None => return None,
        };
        let mut map: HashMap<i32, Vec<f32>> = HashMap::new();
        for i in 0..numTeams{
            let mut tmpvec: Vec<f32> = Vec::new();
            for j in 0..29{
                tmpvec.push(oprc[(i, j)] as f32);
            }
            map.insert(*inverseteammap.get(&(i as isize)).unwrap() as i32, tmpvec);
        }

        return Some(map);
    }).await;
}

pub(crate) async fn getTeams() -> InsightTeamReturn{
    let localCacheConn;
    unsafe{
        localCacheConn = (&cacheConn).as_ref().unwrap();
    }
    let teams: Vec<i32> = localCacheConn.call(|conn|{
        return conn.prepare("SELECT DISTINCT team FROM cone_components").unwrap().query_map([], |row|{
            let val: String = row.get(0).unwrap();
            return (Ok(val.parse::<i32>().unwrap()));
        }).unwrap().collect::<Result<Vec<i32>, rusqlite::Error>>().unwrap();
    }).await;

    let teamsClone = teams.clone();

    let teamNames: HashMap<i32, String> = database::getTeams(2022).await;

    let events: HashMap<i32, Vec<String>> = localCacheConn.call(|conn|{
        let mut map: HashMap<i32, Vec<String>> = HashMap::new();
        for team in teams{
            let events: Vec<String> = conn.prepare(&format!("SELECT DISTINCT eventcode FROM cone_components WHERE team='{}'", team)).unwrap().query_map([], |row|{
                return (Ok(row.get(0).unwrap()));
            }).unwrap().collect::<Result<Vec<String>, rusqlite::Error>>().unwrap();
            map.insert(team, events);
        }
        return map;
    }).await;

    let mut insightTeams: Vec<InsightTeam> = Vec::new();
    for team in teamsClone{
        insightTeams.push(InsightTeam{
            teamNum: team.to_string(),
            teamName: teamNames.get(&team).unwrap().to_string(),
            events: events.get(&team).unwrap().to_vec(),
        });
    }

    return InsightTeamReturn{
        teams: insightTeams,
    };
}

pub(crate) async fn getTeam(team: i32) -> EventConesReturn{
    let localCacheConn;
    unsafe{
        localCacheConn = (&cacheConn).as_ref().unwrap();
    }

    return EventConesReturn { events: localCacheConn.call(move |conn|{
        return conn.prepare(&format!("SELECT eventcode, dcJunctions11, dcJunctions12, dcJunctions13, dcJunctions14, dcJunctions15, dcJunctions21, dcJunctions22, dcJunctions23, dcJunctions24, dcJunctions25, dcJunctions31, dcJunctions32, dcJunctions33, dcJunctions34, dcJunctions35, dcJunctions41, dcJunctions42, dcJunctions43, dcJunctions44, dcJunctions45, dcJunctions51, dcJunctions52, dcJunctions53, dcJunctions54, dcJunctions55 FROM cone_components WHERE team='{}'", team)).unwrap().query_map([], |row|{
            Ok(
                EventCones{
                    eventcode: row.get(0).unwrap(),
                    cones: [
                        row.get(1).unwrap(),
                        row.get(2).unwrap(),
                        row.get(3).unwrap(),
                        row.get(4).unwrap(),
                        row.get(5).unwrap(),
                        row.get(6).unwrap(),
                        row.get(7).unwrap(),
                        row.get(8).unwrap(),
                        row.get(9).unwrap(),
                        row.get(10).unwrap(),
                        row.get(11).unwrap(),
                        row.get(12).unwrap(),
                        row.get(13).unwrap(),
                        row.get(14).unwrap(),
                        row.get(15).unwrap(),
                        row.get(16).unwrap(),
                        row.get(17).unwrap(),
                        row.get(18).unwrap(),
                        row.get(19).unwrap(),
                        row.get(20).unwrap(),
                        row.get(21).unwrap(),
                        row.get(22).unwrap(),
                        row.get(23).unwrap(),
                        row.get(24).unwrap(),
                        row.get(25).unwrap(),
                    ].to_vec(),
                }
            )
        }).unwrap().collect::<Result<Vec<EventCones>, rusqlite::Error>>().unwrap();;
    }).await };
}