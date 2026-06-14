# Retro Weather App - API Demo

This demo shows how the Retro Weather App API endpoints would work with sample responses, demonstrating the complete 90s internet aesthetic.

## Endpoints

### Health Check
```
GET /health
```

**Response:**
```json
{
  "status": "SUCCESS",
  "data": "OK",
  "message": "Retro Weather App is running",
  "timestamp": "2026-06-14T19:07:28.345Z"
}
```

### Current Weather
```
GET /current/Paris
```

**Response:**
```json
{
  "status": "SUCCESS",
  "data": {
    "location": "Paris",
    "temperature": 18.5,
    "humidity": 72,
    "pressure": 1013,
    "description": "Partly cloudy",
    "wind_speed": 4.2,
    "timestamp": "2026-06-14T19:07:28.345Z"
  },
  "message": "Weather data retrieved from API",
  "timestamp": "2026-06-14T19:07:28.345Z"
}
```

### Weather Forecast
```
GET /forecast/London
```

**Response:**
```json
{
  "status": "SUCCESS",
  "data": {
    "location": "London",
    "forecast": [
      {
        "date": "2026-06-15",
        "temp_min": 12.0,
        "temp_max": 20.0,
        "description": "Sunny"
      },
      {
        "date": "2026-06-16",
        "temp_min": 10.0,
        "temp_max": 18.0,
        "description": "Cloudy"
      },
      {
        "date": "2026-06-17",
        "temp_min": 8.0,
        "temp_max": 15.0,
        "description": "Rainy"
      }
    ],
    "timestamp": "2026-06-14T19:07:28.345Z"
  },
  "message": "Weather forecast retrieved",
  "timestamp": "2026-06-14T19:07:28.345Z"
}
```

## Retro Styling Features

The application features:
- CRT screen effects with scanlines
- Terminal-style fonts and color scheme  
- Vintage ASCII art in UI components
- 90s internet-style API responses with retro styling
- Dark mode support for authentic terminal feel
- Responsive design that works on classic 640x480 screens

## How It Would Work

1. **Backend** (Rust/Actix Web)
   - Handles API endpoints for weather data
   - Connects to SQLite database for caching historical data
   - Integrates with OpenWeatherMap API 
   - Caches responses for better performance

2. **Frontend** (Vite/Vue.js)
   - Renders weather information in retro terminal style
   - Displays temperature, humidity, pressure, and wind data
   - Shows 3-day forecasts with vintage styling
   - Features CRT scanline effects and monospace fonts

## Complete Solution

The project would provide:
- Real-time weather data from OpenWeatherMap API
- Historical weather data storage in SQLite database
- Retro terminal interface with vintage design elements
- API endpoints for current weather, forecasts, and health checks
- 90s internet-style responsive design with CRT effects