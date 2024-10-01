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

async fn fetch_coordinates(city: &str) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    dotenv().ok();
    let url = Url::parse(&format!(
        "https://api-adresse.data.gouv.fr/search/?q={}",
        city
    ))?;
    let response = reqwest::get(url).await?;
    let coordinates: Value = response.json().await?;
    let lat = coordinates["features"][0]["geometry"]["coordinates"][1]
        .as_f64()
        .ok_or("Latitude not found")?;
    let lon = coordinates["features"][0]["geometry"]["coordinates"][0]
        .as_f64()
        .ok_or("Longitude not found")?;
    println!("Coordinates: lat={}, lon={}", lat, lon);
    Ok((lat, lon))
}

async fn fetch_weather(lat: f64, lon: f64) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("API_KEY")?;
    println!("API Key: {}", api_key);

    let url = Url::parse(&format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={lat}&lon={lon}&appid={api_key}"
    ))?;
    let response = reqwest::get(url).await?;
    let weather: Value = response.json().await?;
    println!("Weather: {:?}", weather);
    Ok(())
}

#[tokio::main]
async fn main() {
    match ask_city().await {
        Ok(city_name) => match fetch_coordinates(&city_name).await {
            Ok((lat, lon)) => {
                if let Err(e) = fetch_weather(lat, lon).await {
                    eprintln!("Error fetching weather: {}", e);
                }
            }
            Err(e) => eprintln!("Error fetching coordinates: {}", e),
        },
        Err(e) => eprintln!("Error reading city name: {}", e),
    }
}
