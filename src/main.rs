use dotenv::dotenv;
use reqwest::Url;
use serde_json::Value;
use std::env;

async fn ask_city() -> Result<String, Box<dyn std::error::Error>> {
    println!("Enter the city name:");
    let mut city = String::new();
    std::io::stdin().read_line(&mut city)?;
    Ok(city.trim().to_string())
}

async fn fetch_weather(city: &str) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("API_KEY")?;
    println!("API Key: {}", api_key);

    let url = Url::parse(&format!(
        "https://api-adresse.data.gouv.fr/search/?q={}",
        city
    ))?;
    let response = reqwest::get(url).await?;
    let coordinates: Value = response.json().await?;
    println!("Coordinates: {:?}", coordinates);

    Ok(())
}

#[tokio::main]
async fn main() {
    match ask_city().await {
        Ok(city_name) => {
            if let Err(e) = fetch_weather(&city_name).await {
                eprintln!("Error fetching weather: {}", e);
            }
        }
        Err(e) => eprintln!("Error reading city name: {}", e),
    }
}
