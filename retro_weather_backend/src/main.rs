use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use chrono::Utc;
use dotenv::dotenv;
use reqwest::Client;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Mutex;

// ── Response wrapper ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    status: &'static str,
    data: T,
    message: String,
    timestamp: String,
}

fn ok<T: Serialize>(data: T, message: &str) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse {
        status: "SUCCESS",
        data,
        message: message.to_string(),
        timestamp: Utc::now().to_rfc3339(),
    })
}

fn err_json(code: u16, msg: &str) -> HttpResponse {
    let body = serde_json::json!({ "status": "ERROR", "message": msg });
    match code {
        404 => HttpResponse::NotFound().json(body),
        _ => HttpResponse::InternalServerError().json(body),
    }
}

// ── Domain types ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct WeatherData {
    location: String,
    temperature: f64,
    humidity: u32,
    pressure: u32,
    description: String,
    wind_speed: f64,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
struct ForecastDay {
    date: String,
    temp_min: f64,
    temp_max: f64,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct ForecastData {
    location: String,
    forecast: Vec<ForecastDay>,
    timestamp: String,
}

// ── OpenWeatherMap response shapes ───────────────────────────────────────────

#[derive(Deserialize)]
struct OWMCurrentResp {
    main: OWMMain,
    wind: OWMWind,
    weather: Vec<OWMWeatherDesc>,
    name: String,
}

#[derive(Deserialize)]
struct OWMMain {
    temp: f64,
    humidity: u32,
    pressure: u32,
    temp_min: f64,
    temp_max: f64,
}

#[derive(Deserialize)]
struct OWMWind {
    speed: f64,
}

#[derive(Deserialize)]
struct OWMWeatherDesc {
    description: String,
}

#[derive(Deserialize)]
struct OWMForecastResp {
    list: Vec<OWMForecastItem>,
    city: OWMCity,
}

#[derive(Deserialize)]
struct OWMCity {
    name: String,
}

#[derive(Deserialize)]
struct OWMForecastItem {
    dt_txt: String,
    main: OWMMain,
    weather: Vec<OWMWeatherDesc>,
}

// ── App state ────────────────────────────────────────────────────────────────

struct AppState {
    db: Mutex<Connection>,
    client: Client,
    api_key: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn cached(db: &Connection, city: &str, kind: &str, ttl: i64) -> Option<String> {
    let cutoff = Utc::now().timestamp() - ttl;
    db.query_row(
        "SELECT data FROM weather_cache WHERE city=?1 AND type=?2 AND timestamp>?3",
        params![city, kind, cutoff],
        |row| row.get(0),
    )
    .ok()
}

fn store(db: &Connection, city: &str, kind: &str, data: &str) {
    let _ = db.execute(
        "INSERT OR REPLACE INTO weather_cache (city,type,data,timestamp) VALUES (?1,?2,?3,?4)",
        params![city, kind, data, Utc::now().timestamp()],
    );
}

// ── Routes ────────────────────────────────────────────────────────────────────

#[get("/health")]
async fn health() -> HttpResponse {
    ok("OK", "Retro Weather App is running")
}

#[get("/current/{city}")]
async fn current_weather(path: web::Path<String>, state: web::Data<AppState>) -> HttpResponse {
    let city = path.into_inner();
    let key = city.to_lowercase();

    // cache hit (5-min TTL)
    {
        let db = state.db.lock().unwrap();
        if let Some(raw) = cached(&db, &key, "current", 300) {
            if let Ok(w) = serde_json::from_str::<WeatherData>(&raw) {
                return ok(w, "Weather data retrieved from cache");
            }
        }
    }

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, state.api_key
    );

    let resp = match state.client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return err_json(500, "Failed to reach OpenWeatherMap"),
    };

    if !resp.status().is_success() {
        return err_json(404, &format!("City '{}' not found", city));
    }

    let owm = match resp.json::<OWMCurrentResp>().await {
        Ok(d) => d,
        Err(_) => return err_json(500, "Failed to parse weather response"),
    };

    let desc = owm
        .weather
        .first()
        .map(|w| w.description.clone())
        .unwrap_or_default();

    let weather = WeatherData {
        location: owm.name,
        temperature: round1(owm.main.temp),
        humidity: owm.main.humidity,
        pressure: owm.main.pressure,
        description: desc,
        wind_speed: round1(owm.wind.speed),
        timestamp: Utc::now().to_rfc3339(),
    };

    if let Ok(json) = serde_json::to_string(&weather) {
        let db = state.db.lock().unwrap();
        store(&db, &key, "current", &json);
    }

    ok(weather, "Weather data retrieved from API")
}

#[get("/forecast/{city}")]
async fn weather_forecast(path: web::Path<String>, state: web::Data<AppState>) -> HttpResponse {
    let city = path.into_inner();
    let key = city.to_lowercase();

    // cache hit (30-min TTL)
    {
        let db = state.db.lock().unwrap();
        if let Some(raw) = cached(&db, &key, "forecast", 1800) {
            if let Ok(f) = serde_json::from_str::<ForecastData>(&raw) {
                return ok(f, "Forecast retrieved from cache");
            }
        }
    }

    let url = format!(
        "https://api.openweathermap.org/data/2.5/forecast?q={}&appid={}&units=metric",
        city, state.api_key
    );

    let resp = match state.client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return err_json(500, "Failed to reach OpenWeatherMap"),
    };

    if !resp.status().is_success() {
        return err_json(404, &format!("City '{}' not found", city));
    }

    let owm = match resp.json::<OWMForecastResp>().await {
        Ok(d) => d,
        Err(_) => return err_json(500, "Failed to parse forecast response"),
    };

    // Aggregate 3-hour slots into daily min/max
    use std::collections::BTreeMap;
    let mut days: BTreeMap<String, (f64, f64, String)> = BTreeMap::new();

    for item in &owm.list {
        let date = item.dt_txt.split(' ').next().unwrap_or("").to_string();
        if date.is_empty() {
            continue;
        }
        let desc = item
            .weather
            .first()
            .map(|w| w.description.clone())
            .unwrap_or_default();
        days.entry(date)
            .and_modify(|(lo, hi, d)| {
                if item.main.temp_min < *lo {
                    *lo = item.main.temp_min;
                }
                if item.main.temp_max > *hi {
                    *hi = item.main.temp_max;
                }
                *d = desc.clone();
            })
            .or_insert((item.main.temp_min, item.main.temp_max, desc));
    }

    let forecast_days: Vec<ForecastDay> = days
        .into_iter()
        .take(3)
        .map(|(date, (lo, hi, desc))| ForecastDay {
            date,
            temp_min: round1(lo),
            temp_max: round1(hi),
            description: desc,
        })
        .collect();

    let forecast = ForecastData {
        location: owm.city.name,
        forecast: forecast_days,
        timestamp: Utc::now().to_rfc3339(),
    };

    if let Ok(json) = serde_json::to_string(&forecast) {
        let db = state.db.lock().unwrap();
        store(&db, &key, "forecast", &json);
    }

    ok(forecast, "Weather forecast retrieved")
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let api_key = env::var("OPENWEATHER_API_KEY").expect("OPENWEATHER_API_KEY must be set in .env");

    let conn = Connection::open("weather_cache.db").expect("Cannot open SQLite database");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS weather_cache (
            city      TEXT NOT NULL,
            type      TEXT NOT NULL,
            data      TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            PRIMARY KEY (city, type)
        );",
    )
    .expect("Cannot create cache table");

    let state = web::Data::new(AppState {
        db: Mutex::new(conn),
        client: Client::new(),
        api_key,
    });

    println!("╔══════════════════════════════════╗");
    println!("║  RETRO WEATHER SYSTEM v1.0       ║");
    println!("║  http://127.0.0.1:8080           ║");
    println!("╚══════════════════════════════════╝");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .service(health)
            .service(current_weather)
            .service(weather_forecast)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
