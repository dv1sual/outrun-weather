<template>

  <!-- ── Fixed background scene ─────────────────────────────────────────── -->
  <div class="scene" aria-hidden="true">
    <!-- Sun with stripe overlay -->
    <div class="sun"></div>
    <!-- Horizon glow bar -->
    <div class="horizon"></div>
    <!-- Road perspective stripes -->
    <div class="road">
      <div class="road-center"></div>
    </div>
    <!-- Palm tree silhouettes -->
    <div class="palm palm--l">
      <div class="palm-leaf"></div>
      <div class="palm-leaf"></div>
      <div class="palm-leaf"></div>
      <div class="palm-leaf"></div>
      <div class="palm-leaf"></div>
      <div class="palm-leaf"></div>
      <div class="palm-leaf"></div>
      <div class="palm-trunk"></div>
    </div>
    <div class="palm palm--r">
      <div class="palm-leaf"></div>
      <div class="palm-leaf"></div>
      <div class="palm-leaf"></div>
      <div class="palm-leaf"></div>
      <div class="palm-leaf"></div>
      <div class="palm-leaf"></div>
      <div class="palm-leaf"></div>
      <div class="palm-trunk"></div>
    </div>
  </div>

  <!-- ── Scrollable content ──────────────────────────────────────────────── -->
  <div class="screen">

    <!-- Title -->
    <header class="title-wrap">
      <div class="stars">* * * * * * * * * * * * *</div>
      <h1 class="title">
        <span class="title-out">OUT</span><span class="title-run">-RUN</span>
        <br>
        <span class="title-weather">WEATHER</span>
      </h1>
      <div class="subtitle">ATMOSPHERIC RACING TERMINAL</div>
      <div class="clock-row">{{ clock }}<span class="blink">_</span></div>
    </header>

    <!-- Search -->
    <section class="card card--search">
      <div class="card-label">ENTER CITY</div>
      <div class="search-row">
        <span class="prompt">&#9658;</span>
        <input
          ref="inputEl"
          v-model="cityInput"
          class="city-input"
          maxlength="30"
          placeholder="CITY NAME..."
          :disabled="loading"
          @keydown.enter="query"
        />
        <button class="go-btn" :disabled="loading" @click="query">
          {{ loading ? '...' : 'GO!' }}
        </button>
      </div>
    </section>

    <!-- Error -->
    <section v-if="error" class="card card--error">
      <div class="error-text">!! {{ error }}</div>
    </section>

    <!-- Current Weather -->
    <section v-if="weather" class="card card--weather">
      <div class="card-header">
        <span class="dot">&#9679;</span>
        CURRENT
        <span class="city-name">{{ weather.location.toUpperCase() }}</span>
      </div>

      <div class="stats">
        <div class="stat">
          <div class="stat-val yellow">
            {{ weather.temperature.toFixed(1) }}<span class="unit">°C</span>
          </div>
          <div class="stat-lbl">TEMP</div>
        </div>
        <div class="stat">
          <div class="stat-val cyan">
            {{ weather.humidity }}<span class="unit">%</span>
          </div>
          <div class="stat-lbl">HUMID</div>
        </div>
        <div class="stat">
          <div class="stat-val pink">
            {{ weather.pressure }}<span class="unit">hPa</span>
          </div>
          <div class="stat-lbl">PRESS</div>
        </div>
        <div class="stat">
          <div class="stat-val yellow">
            {{ weather.wind_speed.toFixed(1) }}<span class="unit">m/s</span>
          </div>
          <div class="stat-lbl">WIND</div>
        </div>
      </div>

      <div class="condition">
        <span class="cond-arrow">&#9658;</span>
        {{ weather.description.toUpperCase() }}
      </div>
    </section>

    <!-- Forecast -->
    <section v-if="forecast" class="card card--forecast">
      <div class="card-header">
        <span class="dot">&#9679;</span>
        3-DAY FORECAST
        <span class="city-name">{{ forecast.location.toUpperCase() }}</span>
      </div>
      <div class="stages">
        <div
          v-for="(day, i) in forecast.forecast"
          :key="day.date"
          class="stage"
          :class="`stage--${i}`"
        >
          <div class="stage-num">STAGE {{ i + 1 }}</div>
          <div class="stage-date">{{ day.date }}</div>
          <pre class="stage-icon">{{ icon(day.description) }}</pre>
          <div class="stage-desc">{{ day.description.toUpperCase() }}</div>
          <div class="stage-temps">
            <span class="t-lo">&#9660;{{ day.temp_min.toFixed(0) }}°</span>
            <span class="t-sep"> / </span>
            <span class="t-hi">&#9650;{{ day.temp_max.toFixed(0) }}°</span>
          </div>
        </div>
      </div>
    </section>

    <!-- Score footer -->
    <footer class="footer">
      <div class="score-row">
        <span class="score-lbl">SCORE</span>
        <span class="score-val">{{ score }}</span>
        <span class="score-lbl" style="margin-left:2rem">TIME</span>
        <span class="score-val cyan">{{ elapsed }}</span>
      </div>
      <div class="insert-coin">
        <span class="blink">&#9632;</span>
        INSERT COIN TO CONTINUE
        <span class="blink">&#9632;</span>
      </div>
      <div class="credits">POWERED BY OPENWEATHERMAP · RUST/ACTIX-WEB</div>
    </footer>

  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

const cityInput = ref('')
const weather   = ref(null)
const forecast  = ref(null)
const loading   = ref(false)
const error     = ref(null)
const clock     = ref('')
const score     = ref('000000')
const elapsed   = ref('0\'00"')

let ticker, scoreNum = 0, elapsedSec = 0

function tick() {
  const now = new Date()
  clock.value = now.toISOString().slice(0, 19).replace('T', '  ')
  elapsedSec++
  const m = Math.floor(elapsedSec / 60)
  const s = String(elapsedSec % 60).padStart(2, '0')
  elapsed.value = `${m}'${s}"`
}

const inputEl = ref(null)

onMounted(() => {
  tick()
  ticker = setInterval(tick, 1000)
  inputEl.value?.focus()
})

onUnmounted(() => clearInterval(ticker))

async function query() {
  const city = cityInput.value.trim()
  if (!city || loading.value) return

  loading.value  = true
  error.value    = null
  weather.value  = null
  forecast.value = null

  try {
    const [curRes, fcastRes] = await Promise.all([
      fetch(`/api/current/${encodeURIComponent(city)}`),
      fetch(`/api/forecast/${encodeURIComponent(city)}`)
    ])
    const curJson   = await curRes.json()
    const fcastJson = await fcastRes.json()

    if (curJson.status === 'SUCCESS') {
      weather.value = curJson.data
      scoreNum += 5000
      score.value = String(scoreNum).padStart(6, '0')
    } else {
      error.value = curJson.message || 'CITY NOT FOUND'
    }

    if (fcastJson.status === 'SUCCESS') {
      forecast.value = fcastJson.data
    }
  } catch {
    error.value = 'CONNECTION REFUSED - IS THE BACKEND ON PORT 8080?'
  } finally {
    loading.value = false
  }
}

function icon(description) {
  const d = (description || '').toLowerCase()
  if (d.includes('thunder'))    return ' /\\\n/!!\\n \\/ '
  if (d.includes('snow'))       return '* . *\n. * .\n* . *'
  if (d.includes('rain') || d.includes('drizzle'))
                                 return ' ___\n(   )\n|||||'
  if (d.includes('fog') || d.includes('mist') || d.includes('haze'))
                                 return '~~~~~\n~~~~~\n~~~~~'
  if (d.includes('clear') || d.includes('sunny'))
                                 return '\\|/\n-O-\n/|\\'
  if (d.includes('cloud') || d.includes('overcast'))
                                 return ' ___\n(   )\n-----'
  return ' ??? '
}
</script>

<style scoped>
/* ── Fixed scene ─────────────────────────────────────────────────────────── */
.scene {
  position: fixed;
  inset: 0;
  z-index: 0;
  background: linear-gradient(
    to bottom,
    #07000f  0%,
    #0d0021 15%,
    #3a0070 38%,
    #7b2fbe 52%,
    #c4005a 60%,
    #ff2d78 65%,
    #ff6b35 70%,
    #ffe600 73%,
    #ff6b35 75%,
    #1a0030 75%
  );
}

/* Sun */
.sun {
  position: absolute;
  width: 240px;
  height: 240px;
  border-radius: 50%;
  background: #ffe600;
  box-shadow:
    0 0 40px #ff6b35,
    0 0 100px rgba(255,107,53,0.6),
    0 0 180px rgba(255,45,120,0.3);
  top: 57%;
  left: 50%;
  transform: translate(-50%, -50%);
  overflow: hidden;
}

/* Horizontal stripe mask on lower half of sun */
.sun::after {
  content: '';
  position: absolute;
  inset: 0;
  /* dense at bottom, fading out towards top */
  background:
    linear-gradient(to bottom, transparent 35%, rgba(7,0,15,0.15) 50%),
    repeating-linear-gradient(
      to bottom,
      transparent       0px,
      transparent       10px,
      rgba(7,0,15,0.7)  10px,
      rgba(7,0,15,0.7)  14px
    );
  border-radius: 50%;
}

/* Horizon glow */
.horizon {
  position: absolute;
  left: 0; right: 0;
  top: 73%;
  height: 3px;
  background: #ffe600;
  box-shadow: 0 0 30px 8px rgba(255,230,0,0.6), 0 0 80px 20px rgba(255,107,53,0.4);
}

/* Road */
.road {
  position: absolute;
  left: 0; right: 0;
  top: 73%;
  bottom: 0;
  background: repeating-linear-gradient(
    to bottom,
    #ff2d78  0px, #ff2d78  2px,
    #1a0030  2px, #1a0030  16px,
    #ff2d78 16px, #ff2d78  17px,
    #2d0050 17px, #2d0050  38px
  );
  background-size: 100% 38px;
  animation: road-scroll 6s linear infinite;
  overflow: hidden;
}

@keyframes road-scroll {
  from { background-position: 0 0; }
  to   { background-position: 0 38px; }
}

/* Dashed centre line */
.road-center {
  position: absolute;
  left: 50%; top: 0; bottom: 0;
  width: 4px;
  transform: translateX(-50%);
  background: repeating-linear-gradient(
    to bottom,
    #ffe600 0px, #ffe600 12px,
    transparent 12px, transparent 28px
  );
  background-size: 100% 28px;
  animation: road-scroll 6s linear infinite;
  opacity: 0.5;
}

/* Palm trees */
.palm {
  position: absolute;
  bottom: 27%;       /* trunk base sits on the horizon line */
  width: 120px;
  height: 200px;
  transform-origin: bottom center;
  animation: sway 10s ease-in-out infinite;
}
.palm--l { left:  4%; animation-delay: 0s; }
.palm--r { right: 4%; scale: -1 1; animation-delay: -1.8s; }

@keyframes sway {
  0%, 100% { transform: rotate(-2.5deg); }
  50%       { transform: rotate( 2.5deg); }
}

.palm-trunk {
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 10px;
  height: 120px;
  background: #07000f;
  border-radius: 3px;
}

.palm-leaf {
  position: absolute;
  top: 10px;
  left: 50%;
  width: 65px;
  height: 7px;
  background: #07000f;
  border-radius: 0 50% 50% 0;
  transform-origin: 0 50%;
}

.palm-leaf:nth-child(1) { transform: rotate(-75deg); }
.palm-leaf:nth-child(2) { transform: rotate(-50deg); }
.palm-leaf:nth-child(3) { transform: rotate(-25deg); }
.palm-leaf:nth-child(4) { transform: rotate(5deg);   }
.palm-leaf:nth-child(5) { transform: rotate(30deg);  }
.palm-leaf:nth-child(6) { transform: rotate(55deg);  }
.palm-leaf:nth-child(7) { transform: rotate(80deg);  }

/* ── Screen (content) ────────────────────────────────────────────────────── */
.screen {
  position: relative;
  z-index: 1;
  max-width: 820px;
  margin: 0 auto;
  padding: 2rem 1.5rem 4rem;
  font-family: 'Press Start 2P', monospace;
}

/* ── Title ───────────────────────────────────────────────────────────────── */
.title-wrap {
  text-align: center;
  padding: 3rem 0 2rem;
}

.stars {
  color: var(--yellow);
  font-size: 0.55rem;
  letter-spacing: 0.3em;
  margin-bottom: 1.2rem;
  text-shadow: 0 0 8px var(--yellow);
}

.title {
  font-size: 2.4rem;
  line-height: 1.4;
  margin-bottom: 1rem;
  text-shadow:
    3px 3px 0 var(--pink-d),
    0 0 30px rgba(255,45,120,0.8),
    0 0 60px rgba(255,45,120,0.4);
}

.title-out  { color: var(--yellow); }
.title-run  { color: var(--orange); }
.title-weather {
  color: var(--pink);
  font-size: 1.6rem;
}

.subtitle {
  color: var(--cyan);
  font-size: 0.55rem;
  letter-spacing: 0.2em;
  text-shadow: 0 0 10px var(--cyan);
  margin-bottom: 1rem;
}

.clock-row {
  color: var(--pink);
  font-size: 0.6rem;
  text-shadow: 0 0 8px var(--pink);
}

/* ── Cards ───────────────────────────────────────────────────────────────── */
.card {
  border: 2px solid var(--pink);
  background: rgba(7,0,15,0.88);
  padding: 1.25rem 1.5rem;
  margin-bottom: 1.25rem;
  box-shadow:
    0 0 0 1px var(--pink-d),
    0 0 20px rgba(255,45,120,0.25),
    inset 0 0 30px rgba(0,0,0,0.6);
}

.card--search { border-color: var(--cyan); box-shadow: 0 0 20px rgba(0,229,255,0.2), inset 0 0 20px rgba(0,0,0,0.5); }
.card--error  { border-color: var(--orange); }
.card--forecast { border-color: var(--yellow); box-shadow: 0 0 20px rgba(255,230,0,0.15), inset 0 0 20px rgba(0,0,0,0.5); }

.card-label {
  color: var(--cyan);
  font-size: 0.55rem;
  letter-spacing: 0.15em;
  margin-bottom: 0.75rem;
  text-shadow: 0 0 8px var(--cyan);
}

.card-header {
  color: var(--pink);
  font-size: 0.6rem;
  letter-spacing: 0.1em;
  margin-bottom: 1.25rem;
  text-shadow: 0 0 8px var(--pink);
}

.dot { margin-right: 0.5rem; }

.city-name {
  color: var(--yellow);
  margin-left: 0.75rem;
  text-shadow: 0 0 8px var(--yellow);
}

/* ── Search ──────────────────────────────────────────────────────────────── */
.search-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.prompt {
  color: var(--cyan);
  font-size: 0.9rem;
  text-shadow: 0 0 8px var(--cyan);
}

.city-input {
  flex: 1;
  background: transparent;
  border: none;
  border-bottom: 2px solid var(--yellow);
  color: var(--yellow);
  font-family: 'Press Start 2P', monospace;
  font-size: 0.8rem;
  padding: 0.4rem 0.5rem;
  outline: none;
  text-transform: uppercase;
  transition: border-color 0.15s, box-shadow 0.15s;
  min-width: 0;
}

.city-input::placeholder { color: rgba(0,229,255,0.25); }

.city-input:focus {
  border-bottom-color: var(--yellow);
  box-shadow: 0 2px 0 0 rgba(255,230,0,0.3);
}

.city-input:disabled { opacity: 0.4; }

.go-btn {
  background: var(--pink);
  border: 2px solid var(--yellow);
  color: var(--yellow);
  font-family: 'Press Start 2P', monospace;
  font-size: 0.65rem;
  padding: 0.5rem 1rem;
  cursor: pointer;
  text-shadow: 1px 1px 0 #000;
  box-shadow: 3px 3px 0 #000, 0 0 15px rgba(255,45,120,0.5);
  transition: transform 0.08s, box-shadow 0.08s;
  white-space: nowrap;
}

.go-btn:hover:not(:disabled) {
  transform: translate(-1px,-1px);
  box-shadow: 4px 4px 0 #000, 0 0 20px rgba(255,45,120,0.7);
}

.go-btn:active:not(:disabled) {
  transform: translate(2px,2px);
  box-shadow: 1px 1px 0 #000;
}

.go-btn:disabled { opacity: 0.5; cursor: not-allowed; }

/* ── Error ───────────────────────────────────────────────────────────────── */
.error-text {
  color: var(--orange);
  font-size: 0.6rem;
  text-shadow: 0 0 10px var(--orange);
  line-height: 1.8;
}

/* ── Stats ───────────────────────────────────────────────────────────────── */
.stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 1.25rem;
  margin-bottom: 1.25rem;
}

.stat { text-align: center; }

.stat-val {
  font-size: 1.1rem;
  margin-bottom: 0.5rem;
  line-height: 1;
}

.unit {
  font-size: 0.5rem;
  vertical-align: middle;
  margin-left: 0.2rem;
}

.stat-lbl {
  color: rgba(255,255,255,0.4);
  font-size: 0.45rem;
  letter-spacing: 0.15em;
}

.yellow { color: var(--yellow); text-shadow: 0 0 12px rgba(255,230,0,0.7); }
.cyan   { color: var(--cyan);   text-shadow: 0 0 12px rgba(0,229,255,0.7); }
.pink   { color: var(--pink);   text-shadow: 0 0 12px rgba(255,45,120,0.7); }

.condition {
  border-top: 1px solid rgba(255,45,120,0.3);
  padding-top: 1rem;
  color: var(--cyan);
  font-size: 0.6rem;
  letter-spacing: 0.1em;
  text-shadow: 0 0 8px var(--cyan);
}

.cond-arrow {
  color: var(--pink);
  margin-right: 0.75rem;
}

/* ── Forecast stages ─────────────────────────────────────────────────────── */
.stages {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
}

.stage {
  border: 2px solid;
  padding: 0.9rem 0.7rem;
  text-align: center;
  background: rgba(7,0,15,0.6);
}

.stage--0 { border-color: var(--cyan);   box-shadow: 0 0 12px rgba(0,229,255,0.2); }
.stage--1 { border-color: var(--pink);   box-shadow: 0 0 12px rgba(255,45,120,0.2); }
.stage--2 { border-color: var(--yellow); box-shadow: 0 0 12px rgba(255,230,0,0.2); }

.stage-num {
  font-size: 0.45rem;
  letter-spacing: 0.15em;
  margin-bottom: 0.5rem;
  opacity: 0.6;
}

.stage--0 .stage-num { color: var(--cyan); }
.stage--1 .stage-num { color: var(--pink); }
.stage--2 .stage-num { color: var(--yellow); }

.stage-date {
  font-size: 0.45rem;
  color: rgba(255,255,255,0.5);
  margin-bottom: 0.75rem;
}

.stage-icon {
  font-size: 0.65rem;
  line-height: 1.5;
  color: rgba(255,255,255,0.35);
  white-space: pre;
  margin: 0.5rem 0;
  font-family: 'Press Start 2P', monospace;
}

.stage-desc {
  font-size: 0.4rem;
  color: rgba(255,255,255,0.6);
  margin-bottom: 0.75rem;
  line-height: 1.6;
}

.stage-temps { font-size: 0.65rem; }

.t-lo { color: var(--cyan);   text-shadow: 0 0 6px var(--cyan); }
.t-hi { color: var(--orange); text-shadow: 0 0 6px var(--orange); }
.t-sep { color: rgba(255,255,255,0.3); }

/* ── Footer ──────────────────────────────────────────────────────────────── */
.footer {
  margin-top: 2rem;
  border-top: 2px solid rgba(255,45,120,0.3);
  padding-top: 1.5rem;
  text-align: center;
}

.score-row {
  font-size: 0.6rem;
  color: var(--yellow);
  text-shadow: 0 0 8px var(--yellow);
  margin-bottom: 1rem;
  letter-spacing: 0.1em;
}

.score-lbl { color: rgba(255,255,255,0.4); }
.score-val { margin-left: 0.5rem; }

.insert-coin {
  color: var(--pink);
  font-size: 0.5rem;
  letter-spacing: 0.15em;
  text-shadow: 0 0 8px var(--pink);
  margin-bottom: 1rem;
}

.credits {
  color: var(--cyan);
  font-size: 0.4rem;
  letter-spacing: 0.12em;
  text-shadow: 0 0 6px rgba(0,229,255,0.5);
  opacity: 0.75;
}

/* ── Blink ───────────────────────────────────────────────────────────────── */
.blink {
  animation: blink 1s step-end infinite;
  margin: 0 0.5rem;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0; }
}

/* ── Responsive ──────────────────────────────────────────────────────────── */
@media (max-width: 620px) {
  .title { font-size: 1.4rem; }
  .title-weather { font-size: 1rem; }
  .stats { grid-template-columns: repeat(2, 1fr); }
  .stages { grid-template-columns: 1fr; }
}
</style>
