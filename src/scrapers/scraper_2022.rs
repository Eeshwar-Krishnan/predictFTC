use reqwest;
use reqwest::blocking;
use serde_json::{Result as serde_result, Value};
use serde::{Deserialize, Serialize};
use rusqlite::{params, Result};
use tokio_rusqlite::Connection;

use crate::{main, API_KEY};

#[derive(Serialize, Deserialize)]
struct Event{
    eventId: Option<String>,
    code: String,
    divisionCode: Option<String>,
    name: Option<String>,
    remote: bool,
    hybrid: bool,
    fieldCount: i32,
    published: bool,
    _type: Option<String>,
    typeName: Option<String>,
    regionCode: Option<String>,
    leagueCode: Option<String>,
    districtCode: Option<String>,
    venue: Option<String>,
    address: Option<String>,
    city: Option<String>,
    stateprov: Option<String>,
    country: Option<String>,
    website: Option<String>,
    liveStreamUrl: Option<String>,
    webcasts: Option<String>,
    timezone: Option<String>,
    dateStart: Option<String>,
    dateEnd: Option<String>
}

#[derive(Serialize, Deserialize)]
struct EventResponse{
    events: Vec<Event>,
    eventCount: i32
}
#[derive(Serialize, Deserialize)]
struct MatchResultTeam{
    teamNumber: i32,
    station: Option<String>,
    dq: bool,
    onField: bool
}
#[derive(Serialize, Deserialize)]
struct EventMatchResult{
    actualStartTime: Option<String>,
    description: Option<String>,
    tournamentLevel: Option<String>,
    series: i32,
    matchNumber: i32,
    scoreRedFinal: i32,
    scoreRedFoul: i32,
    scoreRedAuto: i32,
    scoreBlueFinal: i32,
    scoreBlueFoul: i32,
    scoreBlueAuto: i32,
    postResultTime: Option<String>,
    teams: Vec<MatchResultTeam>,
    modifiedOn: Option<String>
}
#[derive(Serialize, Deserialize)]
struct EventMatchResponse{
    matches: Vec<EventMatchResult>
}

#[derive(Serialize, Deserialize)]
struct Match_2022_Alliance{
    alliance: Option<String>,
    team: i32,
    sideOfField: Option<String>,
    initSignalSleeve1: bool,
    initSignalSleeve2: bool,
    robot1Auto: String,
    robot2Auto: String,
    autoTerminal: i32,
    autoJunctions: Vec<Vec<Vec<Option<String>>>>,
    dcJunctions: Vec<Vec<Vec<Option<String>>>>,
    dcTerminalNear: i32,
    dcTerminalFar: i32,
    //dcTerminalOther: i32,
    egNavigated1: bool,
    egNavigated2: bool,
    minorPenalties: i32,
    majorPenalties: i32,
    autoNavigationPoints: i32,
    signalBonusPoints: i32,
    autoJunctionConePoints: i32,
    autoTerminalConePoints: i32,
    dcJunctionConePoints: i32,
    dcTerminalConePoints: i32,
    ownershipPoints: i32,
    circuitPoints: i32,
    egNavigationPoints: i32,
    autoPoints: i32,
    dcPoints: i32,
    endgamePoints: i32,
    penaltyPointsCommitted: i32,
    prePenaltyTotal: i32,
    autoJunctionCones: Vec<i32>,
    dcJunctionCones: Vec<i32>,
    beacons: i32,
    ownedJunctions: i32,
    circuit: bool,
    totalPoints: i32,
}
#[derive(Serialize, Deserialize)]
struct Match_Results_2022{
    matchLevel: Option<String>,
    matchSeries: i32,
    matchNumber: i32,
    randomization: i32,
    alliances: Vec<Match_2022_Alliance>
}

#[derive(Serialize, Deserialize)]
struct Match_Result_Response_2022{
    MatchScores: Vec<Match_Results_2022>
}

pub(crate) async fn scrape(conn: Connection){
    let api_key_local;
    unsafe{
        api_key_local = &API_KEY;
    }

    let client = reqwest::blocking::Client::new();

    let mut req = client.get(format!("https://ftc-api.firstinspires.org/v2.0/2022/events"))
                .header("Authorization", format!("Basic {}", api_key_local))
                .header("content-type", "application/json")
                .build().unwrap();
    
    let events_response: EventResponse = serde_json::from_str(&client.execute(req).unwrap().text().unwrap()).unwrap();
    
    static mut results: Vec<Match_Results_2022> = Vec::new();
    static mut matches: Vec<EventMatchResult> = Vec::new();
    static mut codes: Vec<String> = Vec::new();

    for event in events_response.events{
        let mut req = client.get(format!("https://ftc-api.firstinspires.org/v2.0/2022/scores/{}/qual", event.code))
                .header("Authorization", format!("Basic {}", api_key_local))
                .header("content-type", "application/json")
                .build().unwrap();

        let mut result: Match_Result_Response_2022 = serde_json::from_str(&client.execute(req).unwrap().text().unwrap()).unwrap();

        let mut req = client.get(format!("https://ftc-api.firstinspires.org/v2.0/2022/matches/{}", event.code))
                .header("Authorization", format!("Basic {}", api_key_local))
                .header("content-type", "application/json")
                .build().unwrap();

        let mut matchres: EventMatchResponse = serde_json::from_str(&client.execute(req).unwrap().text().unwrap()).unwrap();
        unsafe{
            results.append(&mut result.MatchScores);
            matches.append(&mut matchres.matches);
            codes.push(event.code);
        }
        
    }
    unsafe{
    for index in 0..matches.len(){
            let code = codes.get(index);
            let eventcode = code.unwrap();
            let scores = results.get(index).unwrap();
            let matchval = matches.get(index).unwrap();
            conn.call(move |conn|{
                let blue = scores.alliances.get(0).unwrap();
                let red = scores.alliances.get(1).unwrap();
                let blueres = conn.execute(&format!("INSERT INTO matches{} (eventcode, team1, team2, matchNumber, randomization, alliance, team, sideOfField, initSignalSleeve1, initSignalSleeve2, robot1Auto, robot2Auto, autoTerminal, autoJunctions11, autoJunctions12, autoJunctions13, autoJunctions14, autoJunctions15, autoJunctions21, autoJunctions22, autoJunctions23, autoJunctions24, autoJunctions25, autoJunctions31, autoJunctions32, autoJunctions33, autoJunctions34, autoJunctions35, autoJunctions41, autoJunctions42, autoJunctions43, autoJunctions44, autoJunctions45, autoJunctions51, autoJunctions52, autoJunctions53, autoJunctions54, autoJunctions55, dcJunctions11, dcJunctions12, dcJunctions13, dcJunctions14, dcJunctions15, dcJunctions21, dcJunctions22, dcJunctions23, dcJunctions24, dcJunctions25, dcJunctions31, dcJunctions32, dcJunctions33, dcJunctions34, dcJunctions35, dcJunctions41, dcJunctions42, dcJunctions43, dcJunctions44, dcJunctions45, dcJunctions51, dcJunctions52, dcJunctions53, dcJunctions54, dcJunctions55, dcTerminalNear, dcTerminalFar, egNavigated1, egNavigated2, minorPenalties, majorPenalties, autoNavigationPoints, signalBonusPoints, autoJunctionConePoints, autoTerminalConePoints, dcJunctionConePoints, dcTerminalConePoints, ownershipPoints, circuitPoints, egNavigationPoints, autoPoints, dcPoints, endgamePoints, penaltyPointsCommitted, prePenaltyTotal, autoJunctionConesGnd, autoJunctionConesLow, autoJunctionConesMed, autoJunctionConesHigh, dcJunctionConesGnd, dcJunctionConesLow, dcJunctionConesMed, dcJunctionConesHigh, beacons, ownedJunctions, circuit, totalPoints) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39, ?40, ?41, ?42, ?43, ?44, ?45, ?46, ?47, ?48, ?49, ?50, ?51, ?52, ?53, ?54, ?55, ?56, ?57, ?58, ?59, ?60, ?61, ?62, ?63, ?64, ?65, ?66, ?67, ?68, ?69, ?70, ?71, ?72, ?73, ?74, ?75, ?76, ?77, ?78, ?79, ?80, ?81, ?82, ?83, ?84, ?85, ?86, ?87, ?88, ?89, ?90, ?91, ?92, ?93, ?94, ?95)", "2022"), 
                    params![eventcode,
                            matchval.teams.get(2).unwrap().teamNumber,
                            matchval.teams.get(3).unwrap().teamNumber,
                            scores.matchNumber,
                            scores.randomization,
                            blue.alliance,
                            blue.team,
                            blue.sideOfField,
                            blue.initSignalSleeve1 as i32,
                            blue.initSignalSleeve2 as i32,
                            if (blue.robot1Auto == "NONE") {0} else {if (blue.robot1Auto == "SUBSTATION_TERMINAL") {1} else {2}},
                            if (blue.robot2Auto == "NONE") {0} else {if (blue.robot2Auto == "SUBSTATION_TERMINAL") {1} else {2}},
                            blue.autoTerminal,
                            serializeJunctionArray(blue.autoJunctions.get(0).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(0).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(0).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(0).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(0).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(1).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(1).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(1).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(1).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(1).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(2).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(2).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(2).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(2).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(2).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(3).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(3).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(3).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(3).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(3).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(4).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(4).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(4).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(4).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(blue.autoJunctions.get(4).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(0).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(0).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(0).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(0).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(0).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(1).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(1).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(1).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(1).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(1).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(2).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(2).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(2).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(2).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(2).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(3).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(3).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(3).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(3).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(3).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(4).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(4).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(4).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(4).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(blue.dcJunctions.get(4).unwrap().get(4).unwrap().to_vec()),
                            blue.dcTerminalNear,
                            blue.dcTerminalFar,
                            blue.egNavigated1 as i32,
                            blue.egNavigated2 as i32,
                            blue.minorPenalties,
                            blue.majorPenalties,
                            blue.autoNavigationPoints,
                            blue.signalBonusPoints,
                            blue.autoJunctionConePoints,
                            blue.autoTerminalConePoints,
                            blue.dcJunctionConePoints,
                            blue.dcTerminalConePoints,
                            blue.ownershipPoints,
                            blue.circuitPoints,
                            blue.egNavigationPoints,
                            blue.autoPoints,
                            blue.dcPoints,
                            blue.endgamePoints,
                            blue.penaltyPointsCommitted,
                            blue.prePenaltyTotal,
                            blue.autoJunctionCones.get(0),
                            blue.autoJunctionCones.get(1),
                            blue.autoJunctionCones.get(2),
                            blue.autoJunctionCones.get(3),
                            blue.dcJunctionCones.get(0),
                            blue.dcJunctionCones.get(1),
                            blue.dcJunctionCones.get(2),
                            blue.dcJunctionCones.get(3),
                            blue.beacons,
                            blue.ownedJunctions,
                            blue.circuit,
                            blue.totalPoints,
                        ]).unwrap();

                let redres = conn.execute(&format!("INSERT INTO matches{} (eventcode, team1, team2, matchNumber, randomization, alliance, team, sideOfField, initSignalSleeve1, initSignalSleeve2, robot1Auto, robot2Auto, autoTerminal, autoJunctions11, autoJunctions12, autoJunctions13, autoJunctions14, autoJunctions15, autoJunctions21, autoJunctions22, autoJunctions23, autoJunctions24, autoJunctions25, autoJunctions31, autoJunctions32, autoJunctions33, autoJunctions34, autoJunctions35, autoJunctions41, autoJunctions42, autoJunctions43, autoJunctions44, autoJunctions45, autoJunctions51, autoJunctions52, autoJunctions53, autoJunctions54, autoJunctions55, dcJunctions11, dcJunctions12, dcJunctions13, dcJunctions14, dcJunctions15, dcJunctions21, dcJunctions22, dcJunctions23, dcJunctions24, dcJunctions25, dcJunctions31, dcJunctions32, dcJunctions33, dcJunctions34, dcJunctions35, dcJunctions41, dcJunctions42, dcJunctions43, dcJunctions44, dcJunctions45, dcJunctions51, dcJunctions52, dcJunctions53, dcJunctions54, dcJunctions55, dcTerminalNear, dcTerminalFar, egNavigated1, egNavigated2, minorPenalties, majorPenalties, autoNavigationPoints, signalBonusPoints, autoJunctionConePoints, autoTerminalConePoints, dcJunctionConePoints, dcTerminalConePoints, ownershipPoints, circuitPoints, egNavigationPoints, autoPoints, dcPoints, endgamePoints, penaltyPointsCommitted, prePenaltyTotal, autoJunctionConesGnd, autoJunctionConesLow, autoJunctionConesMed, autoJunctionConesHigh, dcJunctionConesGnd, dcJunctionConesLow, dcJunctionConesMed, dcJunctionConesHigh, beacons, ownedJunctions, circuit, totalPoints) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39, ?40, ?41, ?42, ?43, ?44, ?45, ?46, ?47, ?48, ?49, ?50, ?51, ?52, ?53, ?54, ?55, ?56, ?57, ?58, ?59, ?60, ?61, ?62, ?63, ?64, ?65, ?66, ?67, ?68, ?69, ?70, ?71, ?72, ?73, ?74, ?75, ?76, ?77, ?78, ?79, ?80, ?81, ?82, ?83, ?84, ?85, ?86, ?87, ?88, ?89, ?90, ?91, ?92, ?93, ?94, ?95)", "2022"), 
                    params![eventcode,
                            matchval.teams.get(0).unwrap().teamNumber,
                            matchval.teams.get(1).unwrap().teamNumber,
                            scores.matchNumber,
                            scores.randomization,
                            red.alliance,
                            red.team,
                            red.sideOfField,
                            red.initSignalSleeve1 as i32,
                            red.initSignalSleeve2 as i32,
                            if (red.robot1Auto == "NONE") {0} else {if (red.robot1Auto == "SUBSTATION_TERMINAL") {1} else {2}},
                            if (red.robot2Auto == "NONE") {0} else {if (red.robot2Auto == "SUBSTATION_TERMINAL") {1} else {2}},
                            red.autoTerminal,
                            serializeJunctionArray(red.autoJunctions.get(0).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(0).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(0).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(0).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(0).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(1).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(1).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(1).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(1).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(1).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(2).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(2).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(2).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(2).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(2).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(3).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(3).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(3).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(3).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(3).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(4).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(4).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(4).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(4).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(red.autoJunctions.get(4).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(0).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(0).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(0).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(0).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(0).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(1).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(1).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(1).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(1).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(1).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(2).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(2).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(2).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(2).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(2).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(3).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(3).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(3).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(3).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(3).unwrap().get(4).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(4).unwrap().get(0).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(4).unwrap().get(1).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(4).unwrap().get(2).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(4).unwrap().get(3).unwrap().to_vec()),
                            serializeJunctionArray(red.dcJunctions.get(4).unwrap().get(4).unwrap().to_vec()),
                            red.dcTerminalNear,
                            red.dcTerminalFar,
                            red.egNavigated1 as i32,
                            red.egNavigated2 as i32,
                            red.minorPenalties,
                            red.majorPenalties,
                            red.autoNavigationPoints,
                            red.signalBonusPoints,
                            red.autoJunctionConePoints,
                            red.autoTerminalConePoints,
                            red.dcJunctionConePoints,
                            red.dcTerminalConePoints,
                            red.ownershipPoints,
                            red.circuitPoints,
                            red.egNavigationPoints,
                            red.autoPoints,
                            red.dcPoints,
                            red.endgamePoints,
                            red.penaltyPointsCommitted,
                            red.prePenaltyTotal,
                            red.autoJunctionCones.get(0),
                            red.autoJunctionCones.get(1),
                            red.autoJunctionCones.get(2),
                            red.autoJunctionCones.get(3),
                            red.dcJunctionCones.get(0),
                            red.dcJunctionCones.get(1),
                            red.dcJunctionCones.get(2),
                            red.dcJunctionCones.get(3),
                            red.beacons,
                            red.ownedJunctions,
                            red.circuit,
                            red.totalPoints,
                        ]).unwrap();
            }).await;
    }
    }
}



fn serializeJunctionArray(arr: Vec<Option<String>>) -> String{
    let mut rtnStr = "".to_owned();
    for item in arr{
        let itemval = item.unwrap();
        if(itemval == "MY_CONE"){
            rtnStr += "m";
        }else if(itemval == "OTHER_CONE"){
            rtnStr += "o";
        }
    }
    return rtnStr;
}

