use std::{collections::HashMap, clone, path::Path};

use nalgebra::DMatrix;
use rusqlite::{Statement, Rows};
use tokio_rusqlite::Connection;

static mut cacheConn: Option<Connection> = None;

pub(crate) async fn initialize(path: &Path){
    unsafe{
        cacheConn = Some(Connection::open(path).await.unwrap());
    }
}

pub(crate) async fn updateCache(connect: &Connection){
    let localCacheConn;
    unsafe{
        localCacheConn = (&cacheConn).as_ref().unwrap();
    }

    let events = connect.call(|conn|{
        return conn.prepare("SELECT DISTINCT eventcode FROM 'matches2022'").unwrap().query_map([], |row|{
            return Ok(row.get(0).unwrap());
        }).unwrap().collect::<Result<Vec<String>, rusqlite::Error>>().unwrap();
    }).await;

    for event in events{
        let map = computeEventConeComponents(event.clone(), connect).await.unwrap();
        unsafe{
            localCacheConn.call(move |conn|{
                let localevent = event.clone();
                conn.execute("CREATE TABLE IF NOT EXISTS conecomponents (eventcode, team, dcJunctions11, dcJunctions12, dcJunctions13, dcJunctions14, dcJunctions15, dcJunctions21, dcJunctions22, dcJunctions23, dcJunctions24, dcJunctions25, dcJunctions31, dcJunctions32, dcJunctions33, dcJunctions34, dcJunctions35, dcJunctions41, dcJunctions42, dcJunctions43, dcJunctions44, dcJunctions45, dcJunctions51, dcJunctions52, dcJunctions53, dcJunctions54, dcJunctions55)", []).unwrap();
                conn.prepare("DELETE FROM conecomponents WHERE eventcode=:eventcode").unwrap().execute(&[(":eventcode", &localevent)]).unwrap();
                for teamnum in map.keys(){
                    let arr = map.get(teamnum).unwrap();
                    conn.execute("INSERT INTO conecomponents (eventcode, team, dcJunctions11, dcJunctions12, dcJunctions13, dcJunctions14, dcJunctions15, dcJunctions21, dcJunctions22, dcJunctions23, dcJunctions24, dcJunctions25, dcJunctions31, dcJunctions32, dcJunctions33, dcJunctions34, dcJunctions35, dcJunctions41, dcJunctions42, dcJunctions43, dcJunctions44, dcJunctions45, dcJunctions51, dcJunctions52, dcJunctions53, dcJunctions54, dcJunctions55) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)",
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
        //let mut stmt = conn.prepare("SELECT barcodeElement1,barcodeElement2,carousel,autoNavigated1,autoNavigated2,autoBonus1,autoBonus2,autoStorageFreight,autoFreight1,autoFreight2,autoFreight3,driverControlledStorageFreight,driverControlledFreight1,driverControlledFreight2,driverControlledFreight3,sharedFreight,endgameDelivered,allianceBalanced,sharedUnbalanced,endgameParked1,endgameParked2,capped,minorPenalties,majorPenalties,autoPoints,driverControlledPoints,endgamePoints,penaltyPoints,totalPoints FROM 'inpersonmatches'").unwrap();
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
            //println!("{}, {}: {}", test, teammap.contains_key(&test), eventcode);
            let team1: usize = *teammap.get(&row.get(0).unwrap()).unwrap() as usize;
            let team2: usize = *teammap.get(&row.get(1).unwrap()).unwrap() as usize;

            teammatrix[(team1, team1)] += 1.0;
            teammatrix[(team1, team2)] += 1.0;
            teammatrix[(team2, team1)] += 1.0;
            teammatrix[(team2, team2)] += 1.0;
        }

        //let mut test = RowVector::from(databuffer);

        let mut scorematrix = DMatrix::from_element(numTeams, 29, 0.0);
        let mut scoreRowSize = 0;

        for teamnum in teammap.keys(){
            let mut stmt2: Statement;
            stmt2 = conn.prepare(&format!("SELECT alliance, dcJunctions11, dcJunctions12, dcJunctions13, dcJunctions14, dcJunctions15, dcJunctions21, dcJunctions22, dcJunctions23, dcJunctions24, dcJunctions25, dcJunctions31, dcJunctions32, dcJunctions33, dcJunctions34, dcJunctions35, dcJunctions41, dcJunctions42, dcJunctions43, dcJunctions44, dcJunctions45, dcJunctions51, dcJunctions52, dcJunctions53, dcJunctions54, dcJunctions55 FROM matches2022 WHERE (team1={} OR team2={}) AND eventcode='{}'", teamnum, teamnum, eventcode.clone())).unwrap();
            //let mut stmt2 = conn.prepare(&format!("SELECT totalPoints FROM 'inpersonmatches' WHERE (team1={} OR team2={}) AND eventcode LIKE 'FTCCMP1%'", teamnum, teamnum)).unwrap();
            let mut rows2 = stmt2.query([]).unwrap();
            
            let mut tmparray: [f64; 25] = [0.0; 25];
            while let Some(row) = rows2.next().unwrap(){
                for i in 0..5{
                    
                    for j in 0..5{
                        let alliance: String = row.get(0).unwrap();
                        let index = if(alliance == "Red") {1 + ((i * 5) + j)} else {1 + ((i * 5) + (4-j))};
                        //let index = ((i * 5) + j + 1);
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
        //println!("{}, {}", result.is_invertible(), result.determinant());
        let mut oprc = match(result.solve(&scorematrix)){
            Some(val) => val,
            None => return None,
        };

        let mut data: String = "".to_string();
        data += &inverseteammap.get(&(0 as isize)).unwrap().to_string();
        data += ",";

        for j in 0..29{
            data += &(oprc[(0, j)]).to_string();
            if(j < (29-1)){
                data += ",";
            }
        }
        data += "\n";
        let mut map: HashMap<i32, Vec<f32>> = HashMap::new();
        for i in 0..numTeams{
            let mut tmpvec: Vec<f32> = Vec::new();
            
            data += &inverseteammap.get(&(i as isize)).unwrap().to_string();
            data += ",";

            for j in 0..29{
                data += &(oprc[(i, j)]).to_string();
                tmpvec.push(oprc[(i, j)] as f32);
                if(j < (29-1)){
                    data += ",";
                }
            }
            map.insert(*inverseteammap.get(&(i as isize)).unwrap() as i32, tmpvec);
            data += "\n";
        }

        return Some(map);
    }).await;
}