use serde::{Deserialize, Serialize};

mod PowerPlay;

#[derive(Serialize, Deserialize)]
pub struct Predicted_Match{
    matchNum: i32,
    probability: f32,
    blueWin: bool,
    redWin: bool,
    team1: i32,
    team2: i32,
    team3: i32,
    team4: i32,
    realResult: bool,
    realBlueWin: bool,
    realRedWin: bool,
    predictedcorrect: bool,
}
#[derive(Serialize, Deserialize)]
pub struct Predicted_Matches{
    matches: Vec<Predicted_Match>
}

pub(crate) async fn get_event_predictions(year: String, eventcode: String) -> Option<Predicted_Matches>{
    if(year == "2022"){
        return PowerPlay::get_event_predictions(eventcode).await;
    }else{
        return None;
    }
}