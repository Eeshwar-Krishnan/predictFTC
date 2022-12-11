use std::{collections::HashMap, clone, path::Path, hash::Hash};

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
pub struct EventInsight {
    eventcode: String,
    eventname: String,
    insights: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct EventInsightReturn{
    teams: Vec<EventInsight>
}
//eventcodeOpr, eventcodeDcCone, eventcodeautoCone, eventcodedcOPR, eventcodeautoOPR, eventcodepenOPR, team, opr, dcCone, autoCone, dcOPR, autoOPR, penOPR, oprr, dcConer, autoConer, dcOPRr, autoOPRr, penOPRr, oprdr, dcConedr, autoConedr, dcOPRdr, autoOPRdr, penOPRdr
#[derive(Serialize, Deserialize)]
pub struct TeamInsight {
    teamNum: String,
    teamName: String,
    region: String,
    eventcodes: Vec<String>,
    components: Vec<String>,
    globRankings: Vec<String>,
    districtRankings: Vec<String>,
    numGlobal: i32,
    numDistrict: String
}

#[derive(Serialize, Deserialize)]
pub struct TeamInsightReturn{
    teams: Vec<TeamInsight>
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

    localCacheConn.call(|conn|{
        conn.execute("CREATE TABLE IF NOT EXISTS general_insights (eventcodeOpr, eventcodeDcCone, eventcodeautoCone, eventcodedcOPR, eventcodeautoOPR, eventcodepenOPR, team, teamName, opr, dcCone, autoCone, dcOPR, autoOPR, penOPR, oprr, dcConer, autoConer, dcOPRr, autoOPRr, penOPRr, oprdr, dcConedr, autoConedr, dcOPRdr, autoOPRdr, penOPRdr, district)", []).unwrap();
        conn.prepare("DELETE FROM general_insights").unwrap().execute([]).unwrap();
    }).await;

    let mut bestTeamMap: HashMap<i32, Vec<f32>> = HashMap::new();
    let mut bestEventTeamMap: HashMap<i32, Vec<String>> = HashMap::new();

    for event in events{
        let map = match(computeEventConeComponents(event.clone(), connect).await){
            Some(val) => val,
            None => {
                continue;
            },
        };
        unsafe{
            let localevent = event.clone();
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

            let teamMap = computeEventInsights(localevent.clone(), connect).await.unwrap();

            for teamKey in teamMap.keys(){
                for i in 0..teamMap.get(teamKey).unwrap().len(){
                    if(bestTeamMap.contains_key(teamKey)){
                        if(teamMap.get(teamKey).unwrap().get(i).unwrap() > bestTeamMap.get(teamKey).unwrap().get(i).unwrap()){
                            let mut tmpVec = bestTeamMap.get(teamKey).unwrap().clone();
                            tmpVec[i] = *teamMap.get(teamKey).unwrap().get(i).unwrap();

                            bestTeamMap.insert(*teamKey, tmpVec);

                            let mut tmpVec = bestEventTeamMap.get(teamKey).unwrap().clone();
                            tmpVec[i] = localevent.clone();

                            bestEventTeamMap.insert(*teamKey, tmpVec);
                        }
                    }else{
                        bestTeamMap.insert(*teamKey, teamMap.get(teamKey).unwrap().clone());

                        let mut tmpVec: Vec<String> = Vec::new();
                        for j in 0..teamMap.get(teamKey).unwrap().len(){
                            tmpVec.push(localevent.clone());
                        }

                        bestEventTeamMap.insert(*teamKey, tmpVec);
                    }
                }
            }
        }
    }

    //oprr, dcConer, autoConer, dcOPRr, autoOPRr, penOPRr, oprdr, dcConedr, autoConedr, dcOPRdr, autoOPRdr, penOPRdr

    println!("Processing Global Insights");

    let mut nameMap = database::getTeamNameMap(2022).await;

    let mut districtMap = database::getDistricts(2022).await;

    let mut bestTeamMapClone = bestTeamMap.clone();

    let bestTeamMapRef = &mut bestTeamMap;

    let keys = bestTeamMapRef.clone().into_keys().collect::<Vec<i32>>();

    let mut districtCount: HashMap<String, i32> = HashMap::new();

    for ind1 in 0..bestTeamMapRef.len(){
        if(ind1 % 25 == 0){
            println!("Processed {} of {} team global insights", ind1, bestTeamMapRef.len());
        }

        let team1 = keys[ind1];

        let teamName = nameMap.get(&team1).unwrap().clone();

        let teamDistrict = districtMap.get(&team1).unwrap().clone();

        let mut oprr = 0;
        let mut dcConer = 0;
        let mut autoConer = 0;
        let mut dcOPRr = 0;
        let mut autoOPRr = 0;
        let mut penOPRr = 0;
        let mut oprdr = 0;
        let mut dcConedr = 0;
        let mut autoConedr = 0;
        let mut dcOPRdr = 0;
        let mut autoOPRdr = 0;
        let mut penOPRdr = 0;
        for ind2 in 0..bestTeamMapClone.len(){
            let team2 = bestTeamMapClone.keys().collect::<Vec<&i32>>()[ind2];

            let team1arr = bestTeamMapClone.get(&team1).unwrap();
            let team2arr = bestTeamMapClone.get(team2).unwrap();

            if(team1arr.get(0).unwrap() > team2arr.get(0).unwrap()){
                oprr += 1;
            }
            if(team1arr.get(1).unwrap() > team2arr.get(1).unwrap()){
                dcConer += 1;
            }
            if(team1arr.get(2).unwrap() > team2arr.get(2).unwrap()){
                autoConer += 1;
            }
            if(team1arr.get(3).unwrap() > team2arr.get(3).unwrap()){
                dcOPRr += 1;
            }
            if(team1arr.get(4).unwrap() > team2arr.get(4).unwrap()){
                autoOPRr += 1;
            }
            if(team1arr.get(5).unwrap() > team2arr.get(5).unwrap()){
                penOPRr += 1;
            }

            if(districtMap.get(&team1).unwrap() != districtMap.get(team2).unwrap()){
                continue;
            }

            if(team1arr.get(0).unwrap() > team2arr.get(0).unwrap()){
                oprdr += 1;
            }
            if(team1arr.get(1).unwrap() > team2arr.get(1).unwrap()){
                dcConedr += 1;
            }
            if(team1arr.get(2).unwrap() > team2arr.get(2).unwrap()){
                autoConedr += 1;
            }
            if(team1arr.get(3).unwrap() > team2arr.get(3).unwrap()){
                dcOPRdr += 1;
            }
            if(team1arr.get(4).unwrap() > team2arr.get(4).unwrap()){
                autoOPRdr += 1;
            }
            if(team1arr.get(5).unwrap() > team2arr.get(5).unwrap()){
                penOPRdr += 1;
            }
        }

        let bestEventTeamMapClone = bestEventTeamMap.clone();

        let teameventcodes = vec![bestEventTeamMapClone.get(&team1).unwrap().get(0).unwrap().clone(),
        bestEventTeamMapClone.get(&team1).unwrap().get(1).unwrap().clone(),
        bestEventTeamMapClone.get(&team1).unwrap().get(2).unwrap().clone(),
        bestEventTeamMapClone.get(&team1).unwrap().get(3).unwrap().clone(),
        bestEventTeamMapClone.get(&team1).unwrap().get(4).unwrap().clone(),
        bestEventTeamMapClone.get(&team1).unwrap().get(5).unwrap().clone()];

        let bestTeamsClone = vec![bestTeamMapRef.get(&team1).unwrap().get(0).unwrap().to_string(),
        bestTeamMapRef.get(&team1).unwrap().get(1).unwrap().to_string(),
        bestTeamMapRef.get(&team1).unwrap().get(2).unwrap().to_string(),
        bestTeamMapRef.get(&team1).unwrap().get(3).unwrap().to_string(),
        bestTeamMapRef.get(&team1).unwrap().get(4).unwrap().to_string(),
        bestTeamMapRef.get(&team1).unwrap().get(5).unwrap().to_string(),];

        let team = bestTeamMapRef.get(&team1).unwrap().get(5).unwrap().to_string();

        localCacheConn.call(move  |conn|{
            conn.execute("INSERT INTO general_insights (eventcodeOpr, eventcodeDcCone, eventcodeautoCone, eventcodedcOPR, eventcodeautoOPR, eventcodepenOPR, team, teamName, opr, dcCone, autoCone, dcOPR, autoOPR, penOPR, oprr, dcConer, autoConer, dcOPRr, autoOPRr, penOPRr, oprdr, dcConedr, autoConedr, dcOPRdr, autoOPRdr, penOPRdr, district) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)", 
            [
                teameventcodes.get(0).unwrap(),
                teameventcodes.get(1).unwrap(),
                teameventcodes.get(2).unwrap(),
                teameventcodes.get(3).unwrap(),
                teameventcodes.get(4).unwrap(),
                teameventcodes.get(5).unwrap(),
                &team1.clone().to_string(),
                &teamName,
                bestTeamsClone.get(0).unwrap(),
                bestTeamsClone.get(1).unwrap(),
                bestTeamsClone.get(2).unwrap(),
                bestTeamsClone.get(3).unwrap(),
                bestTeamsClone.get(4).unwrap(),
                bestTeamsClone.get(5).unwrap(),
                &oprr.to_string(),
                &dcConer.to_string(),
                &autoConer.to_string(),
                &dcOPRr.to_string(),
                &autoOPRr.to_string(),
                &penOPRr.to_string(),
                &oprdr.to_string(),
                &dcConedr.to_string(),
                &autoConedr.to_string(),
                &dcOPRdr.to_string(),
                &autoOPRdr.to_string(),
                &penOPRdr.to_string(),
                &teamDistrict.to_string()
            ]).unwrap();
        }).await;

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

async fn computeEventInsights(eventcode: String, connect: &Connection) -> Option<HashMap<i32, Vec<f32>>>{
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
            let team1: usize = *teammap.get(&row.get(0).unwrap()).unwrap() as usize;
            let team2: usize = *teammap.get(&row.get(1).unwrap()).unwrap() as usize;

            teammatrix[(team1, team1)] += 1.0;
            teammatrix[(team1, team2)] += 1.0;
            teammatrix[(team2, team1)] += 1.0;
            teammatrix[(team2, team2)] += 1.0;
        }

        let mut scorematrix = DMatrix::from_element(numTeams, 6, 0.0);//29
        let mut scoreRowSize = 0;

        for teamnum in teammap.keys(){
            let mut stmt2: Statement;
            stmt2 = conn.prepare(&format!("SELECT prePenaltyTotal, dcJunctionCones, autoJunctionCones, dcPoints, autoPoints, penaltyPointsCommitted FROM matches2022 WHERE (team1={} OR team2={}) AND eventcode='{}'", teamnum, teamnum, eventcode.clone())).unwrap();
            let mut rows2 = stmt2.query([]).unwrap();
            
            let mut tmparray: [f64; 6] = [0.0; 6];
            while let Some(row) = rows2.next().unwrap(){
                for k in 0..6{
                    let score: f64 = row.get(k).unwrap();
                    tmparray[k] += score;
                }
            }
            
            for i in 0..6{//6
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
            for j in 0..6{
                tmpvec.push(oprc[(i, j)] as f32);//i, j
            }
            map.insert(*inverseteammap.get(&(i as isize)).unwrap() as i32, tmpvec);
        }

        return Some(map);
    }).await;
}

//Only returns the first 50 results when partial is specified
pub(crate) async fn getTeams(partial: Option<String>) -> TeamInsightReturn{
    let localCacheConn;
    unsafe{
        localCacheConn = ((&cacheConn).as_ref()).unwrap();
    }

    let arr = localCacheConn.call(move |conn2|{
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        conn2.prepare(&format!("SELECT team, teamName, opr, dccone, autoCone FROM general_insights")).unwrap().query_map([], |row|{
            let teamNum: String = row.get(0).unwrap();
            map.insert(teamNum, vec![row.get(1).unwrap(), row.get(2).unwrap(), row.get(3).unwrap(), row.get(4).unwrap()]);
            return (Ok(1));
        }).unwrap().collect::<Result<Vec<i32>, rusqlite::Error>>().unwrap();
        return map;
    }).await;

    let mut insights: Vec<TeamInsight> = Vec::new();
    for team in arr.keys(){
        insights.push(TeamInsight{
            teamNum: team.to_string(),
            teamName: arr.get(team).unwrap().get(0).unwrap().to_string(),
            region: "".to_string(),
            eventcodes: vec![],
            components: vec![arr.get(team).unwrap().get(1).unwrap().to_string(), arr.get(team).unwrap().get(2).unwrap().to_string(), arr.get(team).unwrap().get(3).unwrap().to_string()],
            globRankings: vec![],
            districtRankings: vec![],
            numGlobal: 0,
            numDistrict: 0.to_string(),
        });
    }

    return TeamInsightReturn{
        teams: insights,
    };
}

pub(crate) async fn getTeamGeneralInsights(team: String) -> TeamInsight{
    //let insightTeams = getTeams(partial).await;

    let localCacheConn;
    unsafe{
        localCacheConn = (&cacheConn).as_ref().unwrap();
    }

    let districts = database::getDistricts(2022).await;

    if(!districts.contains_key(&team.parse::<i32>().unwrap())){
        return TeamInsight{
            teamNum: (-1).to_string(),
            teamName: "".to_owned(),
            region: "".to_owned(),
            eventcodes: vec![],
            components: vec![],
            globRankings: vec![],
            districtRankings: vec![],
            numGlobal: 0,
            numDistrict: "0".to_string(),
        }
    }

    return localCacheConn.call(move  |conn|{
        let mut stmt2: Statement;
        stmt2 = conn.prepare(&format!("SELECT COUNT(*) FROM general_insights")).unwrap();
        let mut rows2 = stmt2.query([]).unwrap();
        let count: i32 = rows2.next().unwrap().unwrap().get(0).unwrap();

        let mut stmt2: Statement;
        stmt2 = conn.prepare(&format!("SELECT COUNT(*) FROM general_insights WHERE district='{}'", districts.get(&team.parse::<i32>().unwrap()).unwrap())).unwrap();
        let mut rows2 = stmt2.query([]).unwrap();
        let count2: i32 = rows2.next().unwrap().unwrap().get(0).unwrap();

        let mut stmt2: Statement;
        stmt2 = conn.prepare(&format!("SELECT team, teamName, opr, dcCone, autoCone, dcOPR, autoOPR, penOPR, oprr, dcConer, autoConer, dcOPRr, autoOPRr, penOPRr, oprdr, dcConedr, autoConedr, dcOPRdr, autoOPRdr, penOPRdr FROM general_insights WHERE team=:team")).unwrap();
        let mut rows2 = stmt2.query(&[(":team", team.to_string().as_str())]).unwrap();

        let row = match(rows2.next().unwrap()){
            Some(val) => val,
            None => {
                return TeamInsight{
                    teamNum: (-1).to_string(),
                    teamName: "".to_owned(),
                    region: "".to_owned(),
                    eventcodes: vec![],
                    components: vec![],
                    globRankings: vec![],
                    districtRankings: vec![],
                    numGlobal: 0,
                    numDistrict: "0".to_string(),
                }
            },
        };

        let teamInsight = TeamInsight{
            teamNum: row.get(0).unwrap(),
            teamName: row.get(1).unwrap(),
            eventcodes: vec![],
            components: vec![row.get(2).unwrap(), row.get(3).unwrap(), row.get(4).unwrap(), row.get(5).unwrap(), row.get(6).unwrap(), row.get(7).unwrap()],
            globRankings: vec![row.get(8).unwrap(), row.get(9).unwrap(), row.get(10).unwrap(), row.get(11).unwrap(), row.get(12).unwrap(), row.get(13).unwrap()],
            districtRankings: vec![row.get(14).unwrap(), row.get(15).unwrap(), row.get(16).unwrap(), row.get(17).unwrap(), row.get(18).unwrap(), row.get(19).unwrap()],
            numGlobal: count,
            numDistrict: count2.to_string(),
            region: districts.get(&team.parse::<i32>().unwrap()).unwrap().to_string(),
        };

        return teamInsight;
    }).await;
}

pub(crate) async fn getCones(team: i32) -> EventConesReturn{
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