use tokio_rusqlite::Connection;

pub mod insights_2022;

pub(crate) async fn getTeams(year: String, partial: Option<String>) -> String{
    if(year == "2022"){
        return serde_json::to_string(&insights_2022::getTeams(partial).await).unwrap();
    }
    return "".to_string();
}

pub(crate) async fn getTeam(year: String, team: i32) -> String{
    if(year == "2022"){
        return serde_json::to_string(&insights_2022::getTeamGeneralInsights(team.to_string()).await).unwrap();
    }
    return "".to_string();
}

pub(crate) async fn getCones(year: String, team: i32) -> String{
    if(year == "2022"){
        return serde_json::to_string(&insights_2022::getCones(team).await).unwrap();
    }
    return "".to_string();
}