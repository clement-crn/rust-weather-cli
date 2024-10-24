use dotenv::dotenv;
use reqwest::Url;
use serde_json::Value;
use std::env;

#[derive(Debug)]
struct WeatherInfo {
    city: String,
    temperature: f64,
    icon: String,
    rain_risk: f64,
}

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
    println!("Response: {:?}", response);
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

async fn fetch_weather(
    lat: f64,
    lon: f64,
    city_name: &str,
) -> Result<Vec<WeatherInfo>, Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("API_KEY")?;
    println!("API Key: {}", api_key);

    let url = Url::parse(&format!(
        "https://api.openweathermap.org/data/2.5/forecast?lat={}&lon={}&appid={}&units=metric",
        lat, lon, api_key
    ))?;
    let response = reqwest::get(url).await?;
    let weather: Value = response.json().await?;
    println!("Weather Response: {:?}", weather);

    let list = weather["list"]
        .as_array()
        .ok_or("Forecast list not found")?;
    let mut daily_forecast = Vec::new();

    for entry in list.iter().step_by(8).take(5) {
        let temperature = entry["main"]["temp"].as_f64().unwrap_or(0.0);
        let icon = entry["weather"][0]["icon"]
            .as_str()
            .ok_or("Icon not found")?
            .to_string();
        let rain_risk = entry["rain"]["3h"].as_f64().unwrap_or(0.0);

        daily_forecast.push(WeatherInfo {
            city: city_name.to_string(),
            temperature,
            icon,
            rain_risk,
        });
    }

    Ok(daily_forecast)
}

#[tokio::main]
async fn main() {
    match ask_city().await {
        Ok(city_name) => match fetch_coordinates(&city_name).await {
            Ok((lat, lon)) => match fetch_weather(lat, lon, &city_name).await {
                Ok(weather_forecast) => {
                    for weather in weather_forecast {
                        println!(
                            r#"{{"city": "{}", "temperature": {}, "icon": "{}", "rain_risk": {}}}"#,
                            weather.city, weather.temperature, weather.icon, weather.rain_risk
                        );
                    }
                }
                Err(e) => eprintln!("Error fetching weather: {}", e),
            },
            Err(e) => eprintln!("Error fetching coordinates: {}", e),
        },
        Err(e) => eprintln!("Error reading city name: {}", e),
    }
}
