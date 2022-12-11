use crate::database;

use super::Predicted_Matches;

mod basic_elo_2022;

pub(crate) async fn get_event_predictions(eventcode: String) -> Option<Predicted_Matches>{
    let matches = database::getMatches2022(Some(eventcode.clone()), None).await;
    return Some(basic_elo_2022::getPredictions(matches, eventcode).await);
}