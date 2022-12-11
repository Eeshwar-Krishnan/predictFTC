use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::{scrapers::scraper_2022::Match_2022, API_KEY, predictors::{Predicted_Matches, Predicted_Match}};

#[derive(Serialize, Deserialize)]
struct Hybrid_Match_Response{
    schedule: Vec<Hybrid_Match>
}

#[derive(Serialize, Deserialize)]
struct Hybrid_Match{
    description: Option<String>,
    tournamentLevel: Option<String>,
    series: i32,
    matchNumber: i32,
    startTime: Option<String>,
    actualStartTime: Option<String>,
    postResultTime: Option<String>,
    scoreRedFinal: Option<i32>,
    scoreRedFoul: Option<i32>,
    scoreRedAuto: Option<i32>,
    scoreBlueFinal: Option<i32>,
    scoreBlueFoul: Option<i32>,
    scoreBlueAuto: Option<i32>,
    scoreBlueDriverControlled: Option<i32>,
    scoreBlueEndgame: Option<i32>,
    redWins: bool,
    blueWins: bool,
    teams: Vec<Scheduled_Match_Team>,
    modifiedOn: Option<String>
}

#[derive(Serialize, Deserialize)]
struct Scheduled_Match_Team{
    teamNumber: Option<i32>,
    station: Option<String>,
    surrogate: bool,
    noShow: bool
}


static D:f32 = 400.0;
static K:f32 = 32.0/2.0;

pub(crate) async fn getPredictions(matches: Vec<Match_2022>, eventcode: String) -> Predicted_Matches{
    let api_key_local;
    unsafe{
        api_key_local = &API_KEY;
    }
    
    let mut knownMatches: HashMap<i32, &Match_2022> = HashMap::new();

    matches.iter().for_each(|item|{ knownMatches.insert(item.matchNumber.parse::<i32>().unwrap(), item).unwrap();});

    let client = reqwest::blocking::Client::new();
    

    let mut req = client.get(format!("https://ftc-api.firstinspires.org/v2.0/2022/schedule/{}/qual/hybrid", eventcode))
                .header("Authorization", format!("Basic {}", api_key_local))
                .header("content-type", "application/json")
                .build().unwrap();    
    let matches_response: Hybrid_Match_Response = serde_json::from_str(&client.execute(req).unwrap().text().unwrap()).unwrap();

    println!("Pulled Matches");

    let mut ELOs: HashMap<i32, f32> = HashMap::new();

    let mut predictions: Vec<Predicted_Match> = Vec::new();


    for hybrid_match in matches_response.schedule{
        if(!ELOs.contains_key(&hybrid_match.teams.get(0).unwrap().teamNumber.unwrap())){
            ELOs.insert(hybrid_match.teams.get(0).unwrap().teamNumber.unwrap(), 600.0);
        }
        if(!ELOs.contains_key(&hybrid_match.teams.get(1).unwrap().teamNumber.unwrap())){
            ELOs.insert(hybrid_match.teams.get(1).unwrap().teamNumber.unwrap(), 600.0);
        }
        if(!ELOs.contains_key(&hybrid_match.teams.get(2).unwrap().teamNumber.unwrap())){
            ELOs.insert(hybrid_match.teams.get(2).unwrap().teamNumber.unwrap(), 600.0);
        }
        if(!ELOs.contains_key(&hybrid_match.teams.get(3).unwrap().teamNumber.unwrap())){
            ELOs.insert(hybrid_match.teams.get(3).unwrap().teamNumber.unwrap(), 600.0);
        }

        let mut redELO: f32 = 0.0;
        let mut blueELO: f32 = 0.0;

        let teams = hybrid_match.teams;

        teams.iter().for_each(|team|{
            if(team.station.clone().unwrap().starts_with("Red")){
                redELO += ELOs.get(&team.teamNumber.unwrap()).unwrap();
            }else{
                blueELO += ELOs.get(&team.teamNumber.unwrap()).unwrap();
            }
        });

        redELO = redELO / 2.0;
        blueELO = blueELO / 2.0;

        let Ea = 1.0/(1.0 + 10.0_f32.powf((redELO - blueELO) / D));

        if(hybrid_match.scoreBlueFinal.is_some()){
            let winChange = K * (1.0-Ea);
            let lossChange = K * (1.0 - (1.0 - Ea));

            let mut teamsArr: Vec<&Scheduled_Match_Team> = Vec::new();

            if(hybrid_match.redWins){
                teams.iter().for_each(|team|{
                    let currentELO = ELOs.get(&team.teamNumber.unwrap()).unwrap();
                    if(team.station.clone().unwrap().starts_with("Red")){
                        ELOs.insert(team.teamNumber.unwrap(), currentELO + winChange);
                    }else{
                        ELOs.insert(team.teamNumber.unwrap(), currentELO - lossChange);
                    }
                    teamsArr.push(team);
                });
            }else if (hybrid_match.blueWins){
                teams.iter().for_each(|team|{
                    let currentELO = ELOs.get(&team.teamNumber.unwrap()).unwrap();
                    if(team.station.clone().unwrap().starts_with("Red")){
                        ELOs.insert(team.teamNumber.unwrap(), currentELO - winChange);
                    }else{
                        ELOs.insert(team.teamNumber.unwrap(), currentELO + lossChange);
                    }
                    teamsArr.push(team);
                });
            }else{
                teams.iter().for_each(|team|{
                    teamsArr.push(team);
                });
            }

            let match_pred = Predicted_Match{
                matchNum: hybrid_match.matchNumber,
                probability: Ea,
                blueWin: Ea < 0.5,
                redWin: Ea > 0.5,
                team1: teamsArr.get(0).unwrap().teamNumber.unwrap(),
                team2: teamsArr.get(1).unwrap().teamNumber.unwrap(),
                team3: teamsArr.get(2).unwrap().teamNumber.unwrap(),
                team4: teamsArr.get(3).unwrap().teamNumber.unwrap(),
                realResult: true,
                realBlueWin: hybrid_match.blueWins,
                realRedWin: hybrid_match.redWins,
                predictedcorrect: if(hybrid_match.blueWins){
                    Ea < 0.5
                }else {
                    Ea > 0.5
                },
            };

            predictions.push(match_pred);
        }else{
            let mut teamsArr: Vec<&Scheduled_Match_Team> = Vec::new();

            teams.iter().for_each(|team|{
                teamsArr.push(team);
            });

            let match_pred = Predicted_Match{
                matchNum: hybrid_match.matchNumber,
                probability: Ea,
                blueWin: Ea < 0.5,
                redWin: Ea > 0.5,
                team1: teamsArr.get(0).unwrap().teamNumber.unwrap(),
                team2: teamsArr.get(1).unwrap().teamNumber.unwrap(),
                team3: teamsArr.get(2).unwrap().teamNumber.unwrap(),
                team4: teamsArr.get(3).unwrap().teamNumber.unwrap(),
                realResult: false,
                realBlueWin: false,
                realRedWin: false,
                predictedcorrect: false,
            };

            predictions.push(match_pred);
        }
    }

    return Predicted_Matches{
        matches: predictions,
    }
}