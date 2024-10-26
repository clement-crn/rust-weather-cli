use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use dotenv::dotenv;
use reqwest::Url;
use serde_json::Value;
use std::env;
use std::io;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Terminal;

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

async fn async_main(city: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (lat, lon) = fetch_coordinates(city).await?;
    let weather_forecast = fetch_weather(lat, lon, city).await?;

    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let main_content = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);

            let title = Paragraph::new(city.to_string())
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, main_content[1]);

            let day_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20); 5].as_ref())
                .split(main_content[1]);

            for (i, (day_chunk, forecast)) in
                day_chunks.iter().zip(weather_forecast.iter()).enumerate()
            {
                let day_forecast = Paragraph::new(format!(
                    "Temp: {:.1}°C\nRain: {:.1}mm\nIcon: {}",
                    forecast.temperature, forecast.rain_risk, forecast.icon
                ))
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Day {}", i + 1)),
                );
                f.render_widget(day_forecast, *day_chunk);
            }
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let city_name = ask_city().await?;
    async_main(&city_name).await
}
