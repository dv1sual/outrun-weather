# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this project is

**Outrun Weather** — a retro arcade weather app styled after the 1986 Sega arcade game Out Run. It has two sub-projects that must run simultaneously:

- `retro_weather_backend/` — Rust/Actix-web REST API, port **8080**
- `retro_weather_frontend/` — Vue 3 + Vite SPA, port **5173**

## Running the app

Both processes must be running at the same time.

```bash
# Terminal 1 — backend
cd retro_weather_backend
cargo run

# Terminal 2 — frontend
cd retro_weather_frontend
npm run dev
```

Open `http://localhost:5173`. The frontend proxies `/api/*` → `http://127.0.0.1:8080/*` via Vite's dev server.

If port 8080 is already in use (e.g. a previous run left a process):
```powershell
$pid = (Get-NetTCPConnection -LocalPort 8080).OwningProcess
Stop-Process -Id $pid -Force
```

## Backend

**Single file:** `retro_weather_backend/src/main.rs` — everything lives here.

**Endpoints:**
- `GET /health` — liveness check
- `GET /current/{city}` — current weather (5-min SQLite cache)
- `GET /forecast/{city}` — 3-day forecast aggregated from OWM 3-hour slots (30-min SQLite cache)

**Key design points:**
- `AppState` holds a `Mutex<Connection>` (SQLite), a `reqwest::Client`, and the API key string — shared across requests via `web::Data`.
- Cache table: `weather_cache(city, type, data, timestamp)` with `PRIMARY KEY (city, type)`. `INSERT OR REPLACE` is used on writes.
- OWM responses are deserialised into internal structs, then re-serialised into domain types before caching. The cached JSON is the domain type, not the raw OWM shape.
- Forecast aggregation uses a `BTreeMap<date_string, (min, max, description)>` — BTreeMap keeps days sorted automatically, so `.take(3)` gives the next 3 days.
- All responses follow `ApiResponse<T> { status, data, message, timestamp }`.

**Config:** `retro_weather_backend/.env` — must contain `OPENWEATHER_API_KEY=...`. This file is git-ignored.

**Build:**
```bash
cd retro_weather_backend
cargo build          # dev
cargo build --release  # optimised
```

## Frontend

**Single component:** `retro_weather_frontend/src/App.vue` — all UI, logic, and scoped styles in one file. `src/style.css` holds global CSS variables and base styles only.

**CSS architecture:**
- Global vars in `style.css`: `--pink`, `--cyan`, `--yellow`, `--orange`, `--purple`, `--dark`, `--darker`, `--font` (Press Start 2P).
- The fixed `.scene` div (z-index 0) renders the Out Run background: sky gradient, `.sun` with stripe overlay, `.horizon` glow bar, animated `.road`, and CSS palm tree silhouettes.
- Scrollable `.screen` (z-index 1) sits above the scene with semi-transparent card backgrounds (`rgba(7,0,15,0.88)`) so the scene shows through.
- Road animation: `background-size` + `background-position` keyframes on `.road` and `.road-center` — both must share the same duration to stay in sync.
- Palm sway: `transform-origin: bottom center` + `rotate()` keyframes on `.palm`. The right palm uses the individual `scale: -1 1` property (not `transform: scaleX(-1)`) so the `transform` property stays free for the animation.
- Palm leaves use `nth-child` CSS rules (not Vue `:style` bindings) to avoid re-renders on every clock tick.

**API calls:** `fetch('/api/current/{city}')` and `fetch('/api/forecast/{city}')` fired in parallel via `Promise.all`. The `/api` prefix is stripped by the Vite proxy.

**Build:**
```bash
cd retro_weather_frontend
npm run build    # outputs to dist/
npm run preview  # serves the dist/ build locally
```
