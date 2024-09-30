use dotenv::dotenv;
use reqwest::Url;
use serde_json::Value;
use std::env;

async fn fetch_weather() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("API_KEY")?;
    println!("API Key: {}", api_key);

    // Define an object coordinates that will hold the latitude and longitude of the city
    let url = Url::parse("https://api-adresse.data.gouv.fr/search/?q=soissons")?;
    let response = reqwest::get(url).await?;
    let coordinates: Value = response.json().await?;
    println!("Coordinates: {:?}", coordinates);

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = fetch_weather().await {
        eprintln!("Error fetching weather: {}", e);
    }
}
