use dotenv::dotenv;
use std::env;

fn fetch_weather() {
    dotenv().ok();
    let api_key = env::var("API_KEY").expect("API_KEY not found in .env file");
    println!("API Key: {}", api_key);
}

fn main() {
    fetch_weather();
}
