use std::{path::Path, fs, any::Any, time::Duration};

// Scheduler, and trait for .seconds(), .minutes(), etc.
use clokwerk::{AsyncScheduler, TimeUnits, Job};
// Import week days and WeekDay
use clokwerk::Interval::*;
use std::thread;

pub mod database;
pub mod scrapers;
pub mod insights;
pub mod predictors;
use tide::{Request, security::{CorsMiddleware, Origin}, http::headers::HeaderValue};
use tide_governor::GovernorMiddleware;
use tide::prelude::*;

static mut API_KEY: String = String::new();
static mut DEV_KEYS: Vec<String> = Vec::new();

#[derive(Deserialize)]
#[serde(default)]
struct TeamRequest {
    partial: Option<String>
}
impl Default for TeamRequest {
    fn default() -> Self {
        Self {
            partial: None
        }
    }
}

#[tokio::main]
async fn main(){
    unsafe{
        API_KEY = fs::read_to_string("api_key.txt").unwrap();
        fs::read_to_string("dev_keys.txt").unwrap().split("\n").for_each(|f| {
            DEV_KEYS.push(f.to_string());
        });
    }

    println!("Initializing");

    database::initialize(Path::new("cache/matches.db")).await;
    //database::scrapeAndUpdateAll().await;

    let mut scheduler = AsyncScheduler::new();
    
    scheduler.every(30.minutes())
        .run(|| {database::scrapeAndUpdateAll()});

    tokio::spawn(async move{
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    
    println!("Starting Server...");
    let mut app = tide::new();
    app.at("/v1/:year/insights/teams")
        .with(GovernorMiddleware::per_second(20).unwrap())
        .with(CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false))
        .get(get_insight_teams);
    app.at("v1/:year/insights/team/:team")
        .with(GovernorMiddleware::per_second(10).unwrap())
        .with(CorsMiddleware::new()
            .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
            .allow_origin(Origin::from("*"))
            .allow_credentials(false))
        .get(get_insight_team);
    app.at("v1/:year/insights/cones/:team")
        .with(GovernorMiddleware::per_second(20).unwrap())
        .with(CorsMiddleware::new()
            .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
            .allow_origin(Origin::from("*"))
            .allow_credentials(false))
        .get(get_cones_team);
    app.at("v1/:year/predictions/:eventcode")
        .with(GovernorMiddleware::per_second(2).unwrap())
        .with(CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false))
        .get(get_event_predictions);
    app.at("v1/dev/2022/matches/:eventcode")
        .with(GovernorMiddleware::per_minute(1).unwrap())
        .with(CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false))
        .get(get_matches_2022);
    println!("Started Server!");
    app.listen("127.0.0.1:7244").await.unwrap();
}

async fn get_insight_teams(mut req: Request<()>) -> tide::Result{
    match(req.param("year")){
        Ok(year) => {
            if(year == "2022"){
                unsafe{
                    let page: TeamRequest = req.query()?;
                    return get_json_response(insights::getTeams(year.to_owned(), page.partial).await);
                }
            }else{
                return Ok(tide::Response::builder(400).body("Invalid year entered").build());
            }
        },
        Err(_) => {
            return Ok(tide::Response::builder(400).body("Invalid parameters entered").build());
        },
    }
}

async fn get_insight_team(mut req: Request<()>) -> tide::Result{
    match(req.param("year")){
        Ok(year) => {
            match(req.param("team")){
                Ok(team) => {
                    match(team.parse::<i32>()){
                        Ok(teamnum) => {
                            unsafe{
                                return get_json_response(insights::getTeam(year.to_owned(), teamnum).await);
                            }
                        },
                        Err(_) => {
                            return Ok(tide::Response::builder(400).body("Invalid parameters entered").build());
                        },
                    }
                },
                Err(_) => {
                    return Ok(tide::Response::builder(400).body("Invalid parameters entered").build());
                },
            }
        },
        Err(_) => {
            return Ok(tide::Response::builder(400).body("Invalid parameters entered").build());
        },
    }
}

async fn get_cones_team(mut req: Request<()>) -> tide::Result{
    match(req.param("year")){
        Ok(year) => {
            match(req.param("team")){
                Ok(team) => {
                    match(team.parse::<i32>()){
                        Ok(teamnum) => {
                            unsafe{
                                return get_json_response(insights::getCones(year.to_owned(), teamnum).await);
                            }
                        },
                        Err(_) => {
                            return Ok(tide::Response::builder(400).body("Invalid parameters entered").build());
                        },
                    }
                },
                Err(_) => {
                    return Ok(tide::Response::builder(400).body("Invalid parameters entered").build());
                },
            }
        },
        Err(_) => {
            return Ok(tide::Response::builder(400).body("Invalid parameters entered").build());
        },
    }
}

async fn get_event_predictions(mut req: Request<()>) -> tide::Result{
    match(req.param("year")){
        Ok(year) => {
            match(req.param("eventcode")){
                Ok(eventcode) => {
                    unsafe{
                        return get_json_response(serde_json::to_string(&predictors::get_event_predictions(year.to_string(), eventcode.to_string()).await.unwrap()).unwrap());
                    }
                },
                Err(_) => {
                    return Ok(tide::Response::builder(400).body("Invalid parameters entered").build());
                },
            }
        },
        Err(_) => {
            return Ok(tide::Response::builder(400).body("Invalid parameters entered").build());
        },
    }
}

async fn get_matches_2022(mut req: Request<()>) -> tide::Result{
    match(req.header("Authorization")){
        Some(str) => {
            unsafe{
                if(!DEV_KEYS.contains(&str.as_str().to_string())){
                    return Ok(tide::Response::builder(400).body("Invalid authorization entered").build());
                }
            }
        },
        None => {
            return Ok(tide::Response::builder(400).body("Invalid authorization entered").build());
        },
    }
    match(req.param("eventcode")){
        Ok(eventcode) => {
            unsafe{
                return get_json_response(serde_json::to_string(&database::getMatches2022Response(Some(eventcode.to_string())).await).unwrap());
            }
        },
        Err(_) => {
            return Ok(tide::Response::builder(400).body("Invalid parameters entered").build());
        },
    }
}

fn get_json_response(response: String) -> tide::Result{
    return Ok(tide::Response::builder(200)
    .content_type(tide::http::mime::JSON)
    .body(response)
    .build());
}