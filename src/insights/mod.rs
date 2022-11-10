use tokio_rusqlite::Connection;

pub mod insights_2022;

pub(crate) async fn getTeams(year: String) -> String{
    if(year == "2022"){
        return serde_json::to_string(&insights_2022::getTeams().await).unwrap();
    }
    return "".to_string();
}

pub(crate) async fn getTeam(year: String, team: i32) -> String{
    if(year == "2022"){
        return serde_json::to_string(&insights_2022::getTeam(team).await).unwrap();
    }
    return "".to_string();
}