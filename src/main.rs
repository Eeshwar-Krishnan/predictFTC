use std::{path::Path, fs};

pub mod database;
pub mod scrapers;
pub mod insights;

static mut API_KEY: String = String::new();

#[tokio::main]
async fn main(){
    unsafe{
        API_KEY = fs::read_to_string("api_key.txt").unwrap();
    }

    database::initialize(Path::new("cache/matches.db")).await;
    database::scrapeAndUpdate().await;
    
    println!("test");
}
