// ═══════════════════════════════════════════════════════════════════════
//  FOCUS FOREST v2.0 — Complete Pomodoro Focus App
//  Single-file Rust + HTML/CSS/JS with Google Icons, Particles,
//  Animations, Growing Tree, Audio Controls, Analytics
// ═══════════════════════════════════════════════════════════════════════

use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer};
use chrono::{Datelike, Local, NaiveDate};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════
//  DATA MODELS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Distraction {
    timestamp: String,
    note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FocusSession {
    id: String,
    start_time: String,
    end_time: Option<String>,
    planned_minutes: f64,
    actual_minutes: f64,
    distractions: Vec<Distraction>,
    completed: bool,
    date: String,
}

#[derive(Debug, Serialize)]
struct WeeklyAnalytics {
    daily_minutes: Vec<f64>,
    daily_sessions: Vec<u32>,
    daily_distractions: Vec<u32>,
    day_labels: Vec<String>,
    total_minutes: f64,
    total_sessions: u32,
    total_distractions: u32,
    avg_session_length: f64,
    total_focus_all_time: f64,
    longest_session: f64,
    completion_rate: f64,
    streak_days: u32,
}

#[derive(Deserialize)]
struct StartReq {
    duration_minutes: f64,
}

#[derive(Deserialize)]
struct EndReq {
    session_id: String,
    actual_minutes: f64,
    completed: bool,
}

#[derive(Deserialize)]
struct DistractionReq {
    session_id: String,
    note: String,
}

struct UploadedSound {
    name: String,
    data: Vec<u8>,
}

// ═══════════════════════════════════════════════════════════════════════
//  APP STATE
// ═══════════════════════════════════════════════════════════════════════

struct AppState {
    sessions: Vec<FocusSession>,
    total_focus_minutes: f64,
    sounds: HashMap<String, UploadedSound>,
}

impl AppState {
    fn new() -> Self {
        Self {
            sessions: Vec::new(),
            total_focus_minutes: 0.0,
            sounds: HashMap::new(),
        }
    }

    fn weekly_analytics(&self) -> WeeklyAnalytics {
        let today = Local::now().date_naive();
        let weekday_num = today.weekday().num_days_from_monday() as i64;
        let start_of_week = today - chrono::Duration::days(weekday_num);

        let mut daily_minutes = vec![0.0f64; 7];
        let mut daily_sessions = vec![0u32; 7];
        let mut daily_distractions = vec![0u32; 7];
        let mut week_minutes = 0.0f64;
        let mut week_sessions = 0u32;
        let mut week_distractions = 0u32;
        let mut longest = 0.0f64;
        let mut completed_count = 0u32;
        let mut active_days = std::collections::HashSet::new();

        for session in &self.sessions {
            if let Ok(d) = NaiveDate::parse_from_str(&session.date, "%Y-%m-%d") {
                let diff = (d - start_of_week).num_days();
                if diff >= 0 && diff < 7 {
                    let idx = diff as usize;
                    daily_minutes[idx] += session.actual_minutes;
                    daily_sessions[idx] += 1;
                    daily_distractions[idx] += session.distractions.len() as u32;
                    week_minutes += session.actual_minutes;
                    week_sessions += 1;
                    week_distractions += session.distractions.len() as u32;
                    if session.actual_minutes > longest {
                        longest = session.actual_minutes;
                    }
                    if session.completed {
                        completed_count += 1;
                    }
                    active_days.insert(idx);
                }
            }
        }

        let labels = vec![
            "Mon".into(), "Tue".into(), "Wed".into(),
            "Thu".into(), "Fri".into(), "Sat".into(), "Sun".into(),
        ];

        let avg = if week_sessions > 0 {
            week_minutes / week_sessions as f64
        } else {
            0.0
        };

        let rate = if week_sessions > 0 {
            (completed_count as f64 / week_sessions as f64) * 100.0
        } else {
            0.0
        };

        WeeklyAnalytics {
            daily_minutes,
            daily_sessions,
            daily_distractions,
            day_labels: labels,
            total_minutes: week_minutes,
            total_sessions: week_sessions,
            total_distractions: week_distractions,
            avg_session_length: avg,
            total_focus_all_time: self.total_focus_minutes,
            longest_session: longest,
            completion_rate: rate,
            streak_days: active_days.len() as u32,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  API HANDLERS
// ═══════════════════════════════════════════════════════════════════════

async fn api_start_session(
    data: web::Data<Mutex<AppState>>,
    body: web::Json<StartReq>,
) -> HttpResponse {
    let mut state = data.lock().unwrap();
    let now = Local::now();
    let session = FocusSession {
        id: Uuid::new_v4().to_string(),
        start_time: now.format("%H:%M:%S").to_string(),
        end_time: None,
        planned_minutes: body.duration_minutes,
        actual_minutes: 0.0,
        distractions: Vec::new(),
        completed: false,
        date: now.format("%Y-%m-%d").to_string(),
    };
    let id = session.id.clone();
    state.sessions.push(session);
    HttpResponse::Ok().json(serde_json::json!({"session_id": id}))
}

async fn api_end_session(
    data: web::Data<Mutex<AppState>>,
    body: web::Json<EndReq>,
) -> HttpResponse {
    let mut state = data.lock().unwrap();
    if let Some(s) = state.sessions.iter_mut().find(|s| s.id == body.session_id) {
        s.end_time = Some(Local::now().format("%H:%M:%S").to_string());
        s.actual_minutes = body.actual_minutes;
        s.completed = body.completed;
        state.total_focus_minutes += body.actual_minutes;
    }
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "total": state.total_focus_minutes
    }))
}

async fn api_log_distraction(
    data: web::Data<Mutex<AppState>>,
    body: web::Json<DistractionReq>,
) -> HttpResponse {
    let mut state = data.lock().unwrap();
    if let Some(s) = state.sessions.iter_mut().find(|s| s.id == body.session_id) {
        s.distractions.push(Distraction {
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            note: body.note.clone(),
        });
    }
    HttpResponse::Ok().json(serde_json::json!({"status": "logged"}))
}

async fn api_analytics(data: web::Data<Mutex<AppState>>) -> HttpResponse {
    let state = data.lock().unwrap();
    HttpResponse::Ok().json(state.weekly_analytics())
}

async fn api_sessions(data: web::Data<Mutex<AppState>>) -> HttpResponse {
    let state = data.lock().unwrap();
    HttpResponse::Ok().json(&state.sessions)
}

async fn api_total(data: web::Data<Mutex<AppState>>) -> HttpResponse {
    let state = data.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({"total": state.total_focus_minutes}))
}

async fn api_upload_sound(
    data: web::Data<Mutex<AppState>>,
    mut payload: Multipart,
) -> HttpResponse {
    let mut file_name = String::from("sound.mp3");
    let mut file_bytes: Vec<u8> = Vec::new();

    while let Some(item) = payload.next().await {
        if let Ok(mut field) = item {
            // content_disposition() returns Option<&ContentDisposition> in 0.7
            if let Some(cd) = field.content_disposition() {
                if let Some(fname) = cd.get_filename() {
                    file_name = fname.to_string();
                }
            }
            while let Some(chunk) = field.next().await {
                if let Ok(bytes) = chunk {
                    file_bytes.extend_from_slice(&bytes);
                }
            }
        }
    }

    let id = Uuid::new_v4().to_string();
    let mut state = data.lock().unwrap();
    state.sounds.insert(
        id.clone(),
        UploadedSound {
            name: file_name.clone(),
            data: file_bytes,
        },
    );
    HttpResponse::Ok().json(serde_json::json!({"id": id, "name": file_name}))
}

async fn api_sound_list(data: web::Data<Mutex<AppState>>) -> HttpResponse {
    let state = data.lock().unwrap();
    let list: Vec<serde_json::Value> = state
        .sounds
        .iter()
        .map(|(id, s)| serde_json::json!({"id": id, "name": s.name}))
        .collect();
    HttpResponse::Ok().json(list)
}

async fn api_get_sound(
    data: web::Data<Mutex<AppState>>,
    path: web::Path<String>,
) -> HttpResponse {
    let state = data.lock().unwrap();
    if let Some(sound) = state.sounds.get(path.as_str()) {
        HttpResponse::Ok()
            .content_type("audio/mpeg")
            .body(sound.data.clone())
    } else {
        HttpResponse::NotFound().finish()
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  SERVE FRONTEND
// ═══════════════════════════════════════════════════════════════════════

async fn index_page() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(get_html())
}

// ═══════════════════════════════════════════════════════════════════════
//  MAIN ENTRY POINT
// ═══════════════════════════════════════════════════════════════════════

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(Mutex::new(AppState::new()));

    println!();
    println!("  ╔══════════════════════════════════════════════╗");
    println!("  ║         🌳  Focus Forest  v2.0              ║");
    println!("  ║                                              ║");
    println!("  ║   Open  →  http://localhost:8080             ║");
    println!("  ║   Stop  →  Ctrl+C                           ║");
    println!("  ╚══════════════════════════════════════════════╝");
    println!();

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(index_page))
            .route("/api/session/start", web::post().to(api_start_session))
            .route("/api/session/end", web::post().to(api_end_session))
            .route("/api/session/distraction", web::post().to(api_log_distraction))
            .route("/api/analytics", web::get().to(api_analytics))
            .route("/api/sessions", web::get().to(api_sessions))
            .route("/api/total", web::get().to(api_total))
            .route("/api/sound/upload", web::post().to(api_upload_sound))
            .route("/api/sound/list", web::get().to(api_sound_list))
            .route("/api/sound/{id}", web::get().to(api_get_sound))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

// ═══════════════════════════════════════════════════════════════════════
//  FULL HTML / CSS / JS FRONTEND
//  Google Material Icons, Inter font, particle effects, animations
// ═══════════════════════════════════════════════════════════════════════

fn get_html() -> String {
    r####"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>Focus Forest</title>

<!-- Google Fonts: Inter + Material Symbols -->
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800;900&display=swap" rel="stylesheet">
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Rounded:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200" rel="stylesheet">

<style>
/* ═══════════════════════════════════════════════════════
   CSS RESET & ROOT VARIABLES
   ═══════════════════════════════════════════════════════ */
*{margin:0;padding:0;box-sizing:border-box}
:root{
  --bg:#06090f;
  --s1:#0d1321;
  --s2:#151d30;
  --s3:#1c2842;
  --s4:#243352;
  --g1:#4ade80;
  --g2:#22d3ee;
  --g3:#a78bfa;
  --g4:#f472b6;
  --rd:#f87171;
  --yl:#fbbf24;
  --or:#fb923c;
  --tx:#f1f5f9;
  --tx2:#94a3b8;
  --tx3:#64748b;
  --br:#1e293b;
  --radius:16px;
  --shadow:0 8px 32px rgba(0,0,0,0.4);
}

html{scroll-behavior:smooth}

body{
  font-family:'Inter',system-ui,-apple-system,sans-serif;
  background:var(--bg);
  color:var(--tx);
  min-height:100vh;
  overflow-x:hidden;
  line-height:1.6;
}

/* ═══════════════════════════════════════════════════════
   PARTICLE CANVAS (behind everything)
   ═══════════════════════════════════════════════════════ */
#particles{
  position:fixed;inset:0;z-index:0;pointer-events:none;
}

/* ═══════════════════════════════════════════════════════
   LAYOUT
   ═══════════════════════════════════════════════════════ */
.wrap{
  position:relative;z-index:1;
  max-width:1320px;margin:0 auto;padding:20px;
}

/* ═══════════════════════════════════════════════════════
   HEADER
   ═══════════════════════════════════════════════════════ */
.hdr{
  text-align:center;padding:32px 0 16px;
  animation:fadeSlideDown .8s ease-out;
}
.hdr h1{
  font-size:2.8em;font-weight:900;letter-spacing:-1px;
  background:linear-gradient(135deg,var(--g1),var(--g2),var(--g3),var(--g4));
  background-size:300% 300%;
  -webkit-background-clip:text;-webkit-text-fill-color:transparent;
  animation:gradShift 6s ease infinite;
}
.hdr p{color:var(--tx2);font-size:1.05em;font-weight:400;margin-top:6px}

@keyframes gradShift{
  0%{background-position:0% 50%}
  50%{background-position:100% 50%}
  100%{background-position:0% 50%}
}
@keyframes fadeSlideDown{
  from{opacity:0;transform:translateY(-20px)}
  to{opacity:1;transform:translateY(0)}
}

/* ═══════════════════════════════════════════════════════
   MATERIAL ICON HELPER
   ═══════════════════════════════════════════════════════ */
.mi{
  font-family:'Material Symbols Rounded';
  font-weight:normal;font-style:normal;font-size:24px;
  display:inline-block;line-height:1;
  text-transform:none;letter-spacing:normal;word-wrap:normal;
  white-space:nowrap;direction:ltr;
  -webkit-font-smoothing:antialiased;
  font-variation-settings:'FILL' 1,'wght' 500,'GRAD' 0,'opsz' 24;
  vertical-align:middle;
}
.mi-sm{font-size:18px}
.mi-lg{font-size:32px}
.mi-xl{font-size:48px}

/* ═══════════════════════════════════════════════════════
   TAB NAVIGATION
   ═══════════════════════════════════════════════════════ */
.tabs{
  display:flex;justify-content:center;gap:4px;
  margin:20px auto 28px;
  background:var(--s1);
  border-radius:20px;padding:6px;
  max-width:560px;
  border:1px solid var(--br);
  box-shadow:var(--shadow);
  animation:fadeSlideDown .8s ease-out .1s both;
}
.tab{
  padding:12px 20px;border:none;background:0;
  color:var(--tx2);cursor:pointer;
  border-radius:14px;font-size:.92em;font-weight:600;
  transition:all .35s cubic-bezier(.4,0,.2,1);
  flex:1;
  display:flex;align-items:center;justify-content:center;gap:8px;
  font-family:'Inter',sans-serif;
}
.tab:hover{color:var(--tx);background:var(--s2)}
.tab.on{
  background:linear-gradient(135deg,var(--g1),var(--g2));
  color:var(--bg);font-weight:700;
  box-shadow:0 4px 20px rgba(74,222,128,.35);
  transform:scale(1.02);
}
.tab .mi{font-size:20px}

/* ═══════════════════════════════════════════════════════
   PANELS
   ═══════════════════════════════════════════════════════ */
.pnl{display:none}
.pnl.on{display:block;animation:panelIn .5s ease-out}
@keyframes panelIn{
  from{opacity:0;transform:translateY(12px) scale(.98)}
  to{opacity:1;transform:translateY(0) scale(1)}
}

/* ═══════════════════════════════════════════════════════
   CARDS
   ═══════════════════════════════════════════════════════ */
.crd{
  background:linear-gradient(145deg,var(--s1),var(--s2));
  border:1px solid var(--br);
  border-radius:var(--radius);
  padding:28px;margin-bottom:18px;
  box-shadow:var(--shadow);
  transition:transform .3s,box-shadow .3s;
  position:relative;overflow:hidden;
}
.crd::before{
  content:'';position:absolute;top:0;left:0;right:0;height:1px;
  background:linear-gradient(90deg,transparent,rgba(74,222,128,.3),transparent);
}
.crd:hover{
  transform:translateY(-3px);
  box-shadow:0 12px 40px rgba(0,0,0,.5);
}
.crd h2{
  font-size:1.15em;font-weight:700;margin-bottom:20px;
  display:flex;align-items:center;gap:10px;
  letter-spacing:-.3px;
}
.crd h2 .mi{
  font-size:22px;
  background:linear-gradient(135deg,var(--g1),var(--g2));
  -webkit-background-clip:text;-webkit-text-fill-color:transparent;
}

/* ═══════════════════════════════════════════════════════
   GRID LAYOUT
   ═══════════════════════════════════════════════════════ */
.grid2{display:grid;grid-template-columns:1fr 1fr;gap:18px}
@media(max-width:900px){.grid2{grid-template-columns:1fr}}

/* ═══════════════════════════════════════════════════════
   TIMER RING
   ═══════════════════════════════════════════════════════ */
.tc{display:flex;flex-direction:column;align-items:center;gap:22px}
.trw{position:relative;width:270px;height:270px}
.trw svg{width:100%;height:100%;transform:rotate(-90deg);filter:drop-shadow(0 0 20px rgba(74,222,128,.15))}
.rbg{fill:none;stroke:var(--s3);stroke-width:7}
.rpg{
  fill:none;stroke:url(#tgrad);stroke-width:7;
  stroke-linecap:round;
  stroke-dasharray:754;stroke-dashoffset:0;
  transition:stroke-dashoffset 1s linear;
}
.ttx{
  position:absolute;top:50%;left:50%;
  transform:translate(-50%,-50%);text-align:center;
}
.ttx .tm{
  font-size:3.4em;font-weight:800;
  font-variant-numeric:tabular-nums;letter-spacing:-2px;
  background:linear-gradient(135deg,var(--g1),var(--g2));
  -webkit-background-clip:text;-webkit-text-fill-color:transparent;
}
.ttx .lb{font-size:.88em;color:var(--tx2);margin-top:4px;font-weight:500}

/* Glow pulse when running */
.trw.running .rpg{
  animation:ringPulse 2s ease-in-out infinite;
}
@keyframes ringPulse{
  0%,100%{filter:drop-shadow(0 0 4px rgba(74,222,128,.3))}
  50%{filter:drop-shadow(0 0 16px rgba(74,222,128,.6))}
}

/* ═══════════════════════════════════════════════════════
   DURATION PRESETS
   ═══════════════════════════════════════════════════════ */
.dur{
  display:flex;gap:8px;flex-wrap:wrap;
  justify-content:center;margin-bottom:14px;
}
.dur button{
  padding:8px 18px;
  border:2px solid var(--br);background:0;
  color:var(--tx2);border-radius:12px;
  cursor:pointer;font-size:.88em;font-weight:600;
  transition:all .3s;font-family:'Inter',sans-serif;
}
.dur button:hover,.dur button.on{
  border-color:var(--g1);color:var(--g1);
  background:rgba(74,222,128,.08);
  box-shadow:0 0 12px rgba(74,222,128,.1);
}

.cdur{
  display:flex;align-items:center;gap:10px;
  justify-content:center;margin-bottom:18px;
}
.cdur label{color:var(--tx2);font-size:.9em;font-weight:500}
.cdur input{
  width:76px;padding:8px 12px;
  background:var(--s2);border:1px solid var(--br);
  border-radius:10px;color:var(--tx);
  font-size:1em;text-align:center;
  font-family:'Inter',sans-serif;
  transition:all .3s;
}
.cdur input:focus{
  outline:0;border-color:var(--g1);
  box-shadow:0 0 12px rgba(74,222,128,.15);
}
.cdur span{color:var(--tx2);font-size:.9em}

/* ═══════════════════════════════════════════════════════
   BUTTONS
   ═══════════════════════════════════════════════════════ */
.bg{display:flex;gap:12px;flex-wrap:wrap;justify-content:center}
.b{
  padding:12px 28px;border:none;border-radius:14px;
  cursor:pointer;font-size:.95em;font-weight:700;
  transition:all .3s cubic-bezier(.4,0,.2,1);
  display:inline-flex;align-items:center;gap:8px;
  font-family:'Inter',sans-serif;letter-spacing:-.2px;
}
.bp{
  background:linear-gradient(135deg,var(--g1),#22c55e);
  color:var(--bg);
  box-shadow:0 4px 20px rgba(74,222,128,.35);
}
.bp:hover{transform:translateY(-2px) scale(1.02);box-shadow:0 8px 30px rgba(74,222,128,.5)}
.bp:active{transform:scale(.98)}
.bd{
  background:linear-gradient(135deg,var(--rd),#ef4444);
  color:#fff;
  box-shadow:0 4px 20px rgba(248,113,113,.3);
}
.bd:hover{transform:translateY(-2px) scale(1.02)}
.bs{
  background:var(--s2);color:var(--tx);
  border:1px solid var(--br);
}
.bs:hover{background:var(--s3);transform:translateY(-1px)}
.ba{
  background:linear-gradient(135deg,var(--g3),#8b5cf6);
  color:#fff;
  box-shadow:0 4px 20px rgba(167,139,250,.3);
}
.ba:hover{transform:translateY(-2px) scale(1.02)}
.by{
  background:linear-gradient(135deg,var(--yl),var(--or));
  color:var(--bg);
  box-shadow:0 4px 20px rgba(251,191,36,.3);
}
.by:hover{transform:translateY(-2px) scale(1.02)}

/* ═══════════════════════════════════════════════════════
   TREE CANVAS
   ═══════════════════════════════════════════════════════ */
#treeCanvas{
  border-radius:14px;border:1px solid var(--br);
  display:block;width:100%;max-width:420px;margin:0 auto;
  box-shadow:0 4px 24px rgba(0,0,0,.4);
}
.tinf{
  text-align:center;margin-top:14px;
  color:var(--tx2);font-size:.9em;font-weight:500;
}
.tinf span{color:var(--g1);font-weight:700}
.tinf .lvl{
  display:inline-block;padding:3px 12px;
  background:rgba(74,222,128,.1);
  border:1px solid rgba(74,222,128,.2);
  border-radius:20px;font-size:.82em;
  margin-left:4px;
}

/* ═══════════════════════════════════════════════════════
   AUDIO CONTROLS
   ═══════════════════════════════════════════════════════ */
.atabs{
  display:flex;gap:4px;
  background:var(--s2);border-radius:12px;padding:4px;
}
.atab{
  padding:8px 14px;border:none;background:0;
  color:var(--tx2);cursor:pointer;border-radius:9px;
  font-size:.84em;transition:all .3s;flex:1;text-align:center;
  font-family:'Inter',sans-serif;font-weight:600;
  display:flex;align-items:center;justify-content:center;gap:5px;
}
.atab:hover{color:var(--tx)}
.atab.on{background:var(--g3);color:#fff;box-shadow:0 2px 12px rgba(167,139,250,.3)}
.atab .mi{font-size:16px}
.apnl{display:none}.apnl.on{display:block;animation:fadeIn .3s}
@keyframes fadeIn{from{opacity:0}to{opacity:1}}

.sgrid{
  display:grid;
  grid-template-columns:repeat(auto-fill,minmax(95px,1fr));
  gap:8px;margin-top:10px;
}
.sb{
  padding:10px 6px;
  background:var(--s2);border:1px solid var(--br);
  border-radius:12px;color:var(--tx2);
  cursor:pointer;font-size:.8em;text-align:center;
  transition:all .3s;font-family:'Inter',sans-serif;font-weight:500;
}
.sb:hover,.sb.on{
  border-color:var(--g2);color:var(--g2);
  background:rgba(34,211,238,.06);
  box-shadow:0 0 12px rgba(34,211,238,.1);
  transform:translateY(-2px);
}
.sb .si{font-size:1.5em;display:block;margin-bottom:3px}

.vc{display:flex;align-items:center;gap:12px;margin-top:14px}
.vc .mi{color:var(--g2);font-size:20px}
.vs{
  flex:1;-webkit-appearance:none;
  height:6px;border-radius:3px;
  background:var(--s3);outline:0;
  transition:all .3s;
}
.vs::-webkit-slider-thumb{
  -webkit-appearance:none;width:18px;height:18px;
  border-radius:50%;background:var(--g2);cursor:pointer;
  box-shadow:0 0 10px rgba(34,211,238,.5);
  transition:all .2s;
}
.vs::-webkit-slider-thumb:hover{transform:scale(1.2)}
.vv{color:var(--tx2);font-size:.84em;min-width:36px;font-weight:600}

.li{display:flex;gap:8px;margin-top:10px}
.li input{
  flex:1;padding:10px 14px;
  background:var(--s2);border:1px solid var(--br);
  border-radius:10px;color:var(--tx);font-size:.9em;
  font-family:'Inter',sans-serif;transition:all .3s;
}
.li input:focus{outline:0;border-color:var(--g2);box-shadow:0 0 12px rgba(34,211,238,.12)}

.fi{margin-top:10px}
.fi input[type=file]{
  width:100%;padding:10px;
  background:var(--s2);border:1px dashed var(--br);
  border-radius:10px;color:var(--tx2);
  font-family:'Inter',sans-serif;cursor:pointer;
  transition:all .3s;
}
.fi input[type=file]:hover{border-color:var(--g3);background:var(--s3)}

/* ═══════════════════════════════════════════════════════
   DISTRACTION LOG
   ═══════════════════════════════════════════════════════ */
.di{display:flex;gap:8px;margin-top:10px}
.di input{
  flex:1;padding:10px 14px;
  background:var(--s2);border:1px solid var(--br);
  border-radius:10px;color:var(--tx);font-size:.9em;
  font-family:'Inter',sans-serif;transition:all .3s;
}
.di input:focus{outline:0;border-color:var(--rd);box-shadow:0 0 12px rgba(248,113,113,.12)}
.dl{margin-top:10px;max-height:160px;overflow-y:auto}
.dit{
  padding:10px 14px;
  background:rgba(248,113,113,.06);
  border:1px solid rgba(248,113,113,.15);
  border-left:3px solid var(--rd);
  border-radius:10px;margin-bottom:6px;
  font-size:.85em;display:flex;
  justify-content:space-between;align-items:center;
  animation:slideIn .3s ease-out;
}
@keyframes slideIn{
  from{opacity:0;transform:translateX(-10px)}
  to{opacity:1;transform:translateX(0)}
}
.dit .dn{font-weight:500}
.dit .dt{color:var(--rd);font-weight:600;font-size:.8em}

/* ═══════════════════════════════════════════════════════
   ANALYTICS STATS GRID
   ═══════════════════════════════════════════════════════ */
.ag{
  display:grid;
  grid-template-columns:repeat(auto-fit,minmax(170px,1fr));
  gap:14px;margin-bottom:22px;
}
.sc{
  background:linear-gradient(145deg,var(--s1),var(--s2));
  border:1px solid var(--br);
  border-radius:var(--radius);padding:20px;
  text-align:center;transition:all .4s;
  position:relative;overflow:hidden;
}
.sc::after{
  content:'';position:absolute;inset:0;
  background:linear-gradient(135deg,transparent 40%,rgba(255,255,255,.02));
  pointer-events:none;
}
.sc:hover{
  transform:translateY(-4px) scale(1.02);
  box-shadow:0 8px 30px rgba(0,0,0,.4);
}
.sc .ico{font-size:28px;margin-bottom:8px;display:block}
.sc .v{font-size:2em;font-weight:800;margin-bottom:4px;letter-spacing:-1px}
.sc .l{font-size:.82em;color:var(--tx2);font-weight:500}
.sc:nth-child(1) .v{color:var(--g1)}
.sc:nth-child(1) .ico{color:var(--g1)}
.sc:nth-child(2) .v{color:var(--g2)}
.sc:nth-child(2) .ico{color:var(--g2)}
.sc:nth-child(3) .v{color:var(--g3)}
.sc:nth-child(3) .ico{color:var(--g3)}
.sc:nth-child(4) .v{color:var(--rd)}
.sc:nth-child(4) .ico{color:var(--rd)}
.sc:nth-child(5) .v{color:var(--yl)}
.sc:nth-child(5) .ico{color:var(--yl)}
.sc:nth-child(6) .v{color:var(--g4)}
.sc:nth-child(6) .ico{color:var(--g4)}
.sc:nth-child(7) .v{color:var(--or)}
.sc:nth-child(7) .ico{color:var(--or)}
.sc:nth-child(8) .v{color:#34d399}
.sc:nth-child(8) .ico{color:#34d399}

/* ═══════════════════════════════════════════════════════
   CHARTS
   ═══════════════════════════════════════════════════════ */
.cc{
  background:linear-gradient(145deg,var(--s1),var(--s2));
  border-radius:var(--radius);padding:24px;
  border:1px solid var(--br);margin-bottom:18px;
  box-shadow:var(--shadow);
}
.cc h3{
  margin-bottom:18px;font-size:1.05em;font-weight:700;
  display:flex;align-items:center;gap:8px;
  letter-spacing:-.3px;
}
.cc h3 .mi{
  font-size:20px;
  background:linear-gradient(135deg,var(--g1),var(--g2));
  -webkit-background-clip:text;-webkit-text-fill-color:transparent;
}
.bc{
  display:flex;align-items:flex-end;
  justify-content:space-around;
  height:200px;gap:10px;padding:0 8px;
}
.bgrp{
  display:flex;flex-direction:column;
  align-items:center;gap:8px;flex:1;
}
.bar{
  width:100%;max-width:48px;min-height:4px;
  border-radius:8px 8px 4px 4px;
  transition:height .8s cubic-bezier(.4,0,.2,1);
  position:relative;
}
.bar.g1{background:linear-gradient(180deg,var(--g1),var(--g2));box-shadow:0 0 12px rgba(74,222,128,.2)}
.bar.g2{background:linear-gradient(180deg,var(--g3),var(--g2));box-shadow:0 0 12px rgba(167,139,250,.2)}
.bar.g3{background:linear-gradient(180deg,var(--rd),var(--or));box-shadow:0 0 12px rgba(248,113,113,.2)}
.bv{font-size:.75em;font-weight:700}
.bl{font-size:.8em;color:var(--tx2);font-weight:600}

/* ═══════════════════════════════════════════════════════
   SESSION HISTORY
   ═══════════════════════════════════════════════════════ */
.sl{max-height:500px;overflow-y:auto}
.si2{
  display:flex;justify-content:space-between;align-items:center;
  padding:16px 20px;
  background:linear-gradient(145deg,var(--s2),var(--s3));
  border:1px solid var(--br);border-radius:14px;
  margin-bottom:8px;transition:all .3s;
}
.si2:hover{border-color:var(--g1);transform:translateX(4px)}
.sinf{display:flex;flex-direction:column;gap:3px}
.sd{font-size:.84em;color:var(--tx2);font-weight:500;display:flex;align-items:center;gap:5px}
.sd .mi{font-size:14px}
.st2{font-weight:700;color:var(--g1);font-size:1.05em}
.sm2{font-size:.8em;color:var(--tx3);display:flex;align-items:center;gap:4px}
.sm2 .mi{font-size:14px;color:var(--rd)}
.ss{
  padding:5px 14px;border-radius:20px;
  font-size:.8em;font-weight:700;
  display:flex;align-items:center;gap:4px;
}
.ss.c{background:rgba(74,222,128,.1);color:var(--g1);border:1px solid rgba(74,222,128,.2)}
.ss.ic{background:rgba(248,113,113,.1);color:var(--rd);border:1px solid rgba(248,113,113,.2)}
.ss .mi{font-size:16px}

/* Empty state */
.empty{
  text-align:center;padding:48px 20px;color:var(--tx2);
}
.empty .mi{font-size:64px;display:block;margin-bottom:16px;opacity:.3}
.empty p{font-size:1.1em;font-weight:500}

/* ═══════════════════════════════════════════════════════
   TOAST NOTIFICATIONS
   ═══════════════════════════════════════════════════════ */
.toast{
  position:fixed;bottom:28px;right:28px;
  padding:14px 22px;border-radius:14px;
  color:#fff;font-weight:600;font-size:.92em;
  z-index:999;
  box-shadow:0 8px 30px rgba(0,0,0,.5);
  display:flex;align-items:center;gap:10px;
  animation:toastIn .4s cubic-bezier(.4,0,.2,1),toastOut .4s 2.6s forwards;
  font-family:'Inter',sans-serif;
  backdrop-filter:blur(10px);
}
.toast.ok{background:linear-gradient(135deg,rgba(34,197,94,.9),rgba(22,163,74,.9))}
.toast.nfo{background:linear-gradient(135deg,rgba(34,211,238,.9),rgba(8,145,178,.9))}
.toast.err{background:linear-gradient(135deg,rgba(248,113,113,.9),rgba(220,38,38,.9))}
.toast .mi{font-size:20px}
@keyframes toastIn{
  from{transform:translateY(80px) scale(.8);opacity:0}
  to{transform:translateY(0) scale(1);opacity:1}
}
@keyframes toastOut{
  from{transform:translateY(0) scale(1);opacity:1}
  to{transform:translateY(80px) scale(.8);opacity:0}
}

/* ═══════════════════════════════════════════════════════
   SCROLLBAR
   ═══════════════════════════════════════════════════════ */
::-webkit-scrollbar{width:6px}
::-webkit-scrollbar-track{background:transparent}
::-webkit-scrollbar-thumb{background:var(--s3);border-radius:3px}
::-webkit-scrollbar-thumb:hover{background:var(--s4)}

/* ═══════════════════════════════════════════════════════
   RESPONSIVE
   ═══════════════════════════════════════════════════════ */
@media(max-width:640px){
  .hdr h1{font-size:2em}
  .trw{width:220px;height:220px}
  .ttx .tm{font-size:2.6em}
  .ag{grid-template-columns:repeat(2,1fr)}
  .tab{padding:10px 8px;font-size:.82em}
  .tab .mi{font-size:18px}
  .crd{padding:20px}
}
</style>
</head>
<body>

<!-- Particle Canvas -->
<canvas id="particles"></canvas>

<div class="wrap">

  <!-- Header -->
  <div class="hdr">
    <h1><span class="mi mi-xl">eco</span> Focus Forest</h1>
    <p>Grow your forest, one focus session at a time</p>
  </div>

  <!-- Tabs -->
  <div class="tabs">
    <button class="tab on" onclick="stab('timer',this)">
      <span class="mi">timer</span>Timer
    </button>
    <button class="tab" onclick="stab('stats',this)">
      <span class="mi">analytics</span>Analytics
    </button>
    <button class="tab" onclick="stab('hist',this)">
      <span class="mi">history</span>Sessions
    </button>
  </div>

  <!-- ══════════════════════════════════════════════════
       TIMER PANEL
       ══════════════════════════════════════════════════ -->
  <div id="p-timer" class="pnl on">
    <div class="grid2">

      <!-- Left Column: Timer + Distractions -->
      <div>
        <div class="crd">
          <h2><span class="mi">timer</span> Focus Timer</h2>

          <div class="dur" id="durBtns">
            <button onclick="sd(15,this)">15m</button>
            <button class="on" onclick="sd(25,this)">25m</button>
            <button onclick="sd(45,this)">45m</button>
            <button onclick="sd(60,this)">60m</button>
            <button onclick="sd(90,this)">90m</button>
            <button onclick="sd(120,this)">120m</button>
          </div>

          <div class="cdur">
            <label>Custom:</label>
            <input type="number" id="cmin" min="1" max="480" value="25"
              onchange="sd(parseInt(this.value))">
            <span>min</span>
          </div>

          <div class="tc">
            <div class="trw" id="timerRing">
              <svg viewBox="0 0 270 270">
                <defs>
                  <linearGradient id="tgrad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" style="stop-color:#4ade80"/>
                    <stop offset="50%" style="stop-color:#22d3ee"/>
                    <stop offset="100%" style="stop-color:#a78bfa"/>
                  </linearGradient>
                </defs>
                <circle class="rbg" cx="135" cy="135" r="120"/>
                <circle class="rpg" id="ring" cx="135" cy="135" r="120"/>
              </svg>
              <div class="ttx">
                <div class="tm" id="tdisp">25:00</div>
                <div class="lb" id="tlbl">Ready to focus</div>
              </div>
            </div>

            <div class="bg">
              <button class="b bp" id="sbtn" onclick="toggle()">
                <span class="mi">play_arrow</span> Start
              </button>
              <button class="b bd" id="rbtn" onclick="rst()" style="display:none">
                <span class="mi">restart_alt</span> Reset
              </button>
            </div>
          </div>
        </div>

        <!-- Distraction Card -->
        <div class="crd" id="dcrd" style="display:none">
          <h2><span class="mi">warning</span> Log Distraction</h2>
          <div class="di">
            <input type="text" id="dnote" placeholder="What distracted you?"
              onkeydown="if(event.key==='Enter')ld()">
            <button class="b bd" onclick="ld()">
              <span class="mi mi-sm">add</span> Log
            </button>
          </div>
          <div class="dl" id="dlist"></div>
        </div>
      </div>

      <!-- Right Column: Tree + Audio -->
      <div>
        <!-- Tree Card -->
        <div class="crd">
          <h2><span class="mi">park</span> Your Tree</h2>
          <canvas id="treeCanvas" width="420" height="360"></canvas>
          <div class="tinf">
            <span class="mi mi-sm">schedule</span>
            Total focus: <span id="tfdisp">0</span> min
            <span class="lvl" id="tlvl">Seed</span>
          </div>
        </div>

        <!-- Audio Card -->
        <div class="crd">
          <h2><span class="mi">music_note</span> Sounds &amp; Music</h2>

          <div class="atabs">
            <button class="atab on" onclick="satab('amb',this)">
              <span class="mi mi-sm">cloud</span> Ambient
            </button>
            <button class="atab" onclick="satab('lnk',this)">
              <span class="mi mi-sm">link</span> Link
            </button>
            <button class="atab" onclick="satab('upl',this)">
              <span class="mi mi-sm">upload_file</span> Upload
            </button>
          </div>

          <!-- Ambient Sounds -->
          <div id="a-amb" class="apnl on">
            <div class="sgrid">
              <button class="sb" onclick="pamb('rain',this)"><span class="si">🌧</span>Rain</button>
              <button class="sb" onclick="pamb('forest',this)"><span class="si">🌲</span>Forest</button>
              <button class="sb" onclick="pamb('ocean',this)"><span class="si">🌊</span>Ocean</button>
              <button class="sb" onclick="pamb('fire',this)"><span class="si">🔥</span>Fire</button>
              <button class="sb" onclick="pamb('wind',this)"><span class="si">💨</span>Wind</button>
              <button class="sb" onclick="pamb('birds',this)"><span class="si">🐦</span>Birds</button>
              <button class="sb" onclick="pamb('cafe',this)"><span class="si">☕</span>Cafe</button>
              <button class="sb" onclick="samb()"><span class="si">🔇</span>Stop</button>
            </div>
          </div>

          <!-- Link Input -->
          <div id="a-lnk" class="apnl">
            <div class="li">
              <input type="url" id="aurl" placeholder="Paste MP3 / audio URL...">
              <button class="b ba" onclick="plnk()">
                <span class="mi mi-sm">play_arrow</span>
              </button>
            </div>
          </div>

          <!-- Upload -->
          <div id="a-upl" class="apnl">
            <div class="fi">
              <input type="file" id="afile" accept="audio/*" onchange="uaud()">
            </div>
            <div id="ulist" style="margin-top:10px"></div>
          </div>

          <!-- Volume -->
          <div class="vc">
            <span class="mi" id="volIcon">volume_up</span>
            <input type="range" class="vs" id="vol" min="0" max="100" value="50"
              oninput="svol(this.value)">
            <span class="vv" id="vv">50%</span>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- ══════════════════════════════════════════════════
       ANALYTICS PANEL
       ══════════════════════════════════════════════════ -->
  <div id="p-stats" class="pnl">
    <div class="ag" id="sgrid"></div>
    <div class="cc">
      <h3><span class="mi">bar_chart</span> Daily Focus Minutes</h3>
      <div class="bc" id="ch1"></div>
    </div>
    <div class="cc">
      <h3><span class="mi">show_chart</span> Sessions Per Day</h3>
      <div class="bc" id="ch2"></div>
    </div>
    <div class="cc">
      <h3><span class="mi">warning</span> Distractions Per Day</h3>
      <div class="bc" id="ch3"></div>
    </div>
  </div>

  <!-- ══════════════════════════════════════════════════
       SESSIONS PANEL
       ══════════════════════════════════════════════════ -->
  <div id="p-hist" class="pnl">
    <div class="crd">
      <h2><span class="mi">history</span> Session History</h2>
      <div class="sl" id="slist"></div>
    </div>
  </div>
</div>

<audio id="apl" loop></audio>

<script>
// ═══════════════════════════════════════════════════════════════
//  PARTICLE SYSTEM
// ═══════════════════════════════════════════════════════════════
(function(){
  const cv=document.getElementById('particles');
  const c=cv.getContext('2d');
  let W,H,particles=[];
  function resize(){W=cv.width=window.innerWidth;H=cv.height=window.innerHeight}
  window.addEventListener('resize',resize);resize();

  class Particle{
    constructor(){this.reset()}
    reset(){
      this.x=Math.random()*W;
      this.y=Math.random()*H;
      this.vx=(Math.random()-.5)*.3;
      this.vy=(Math.random()-.5)*.3;
      this.r=Math.random()*2+.5;
      this.alpha=Math.random()*.4+.1;
      const colors=['74,222,128','34,211,238','167,139,250','244,114,182'];
      this.color=colors[Math.floor(Math.random()*colors.length)];
      this.pulse=Math.random()*Math.PI*2;
      this.pulseSpeed=Math.random()*.02+.005;
    }
    update(){
      this.x+=this.vx;this.y+=this.vy;
      this.pulse+=this.pulseSpeed;
      if(this.x<0||this.x>W||this.y<0||this.y>H)this.reset();
    }
    draw(){
      const a=this.alpha*(0.5+0.5*Math.sin(this.pulse));
      c.beginPath();
      c.arc(this.x,this.y,this.r,0,Math.PI*2);
      c.fillStyle='rgba('+this.color+','+a+')';
      c.fill();
      // glow
      c.beginPath();
      c.arc(this.x,this.y,this.r*3,0,Math.PI*2);
      c.fillStyle='rgba('+this.color+','+(a*.15)+')';
      c.fill();
    }
  }

  for(let i=0;i<80;i++)particles.push(new Particle());

  function drawLines(){
    for(let i=0;i<particles.length;i++){
      for(let j=i+1;j<particles.length;j++){
        const dx=particles[i].x-particles[j].x;
        const dy=particles[i].y-particles[j].y;
        const d=Math.sqrt(dx*dx+dy*dy);
        if(d<120){
          c.beginPath();
          c.moveTo(particles[i].x,particles[i].y);
          c.lineTo(particles[j].x,particles[j].y);
          c.strokeStyle='rgba(74,222,128,'+(0.06*(1-d/120))+')';
          c.lineWidth=.5;
          c.stroke();
        }
      }
    }
  }

  function animate(){
    c.clearRect(0,0,W,H);
    particles.forEach(p=>{p.update();p.draw()});
    drawLines();
    requestAnimationFrame(animate);
  }
  animate();
})();

// ═══════════════════════════════════════════════════════════════
//  TIMER STATE
// ═══════════════════════════════════════════════════════════════
const CIRC=2*Math.PI*120;
let dur=25*60,tl=dur,iv=null,run=false,sid=null,dists=[],elapsed=0,tfm=0;
const ringEl=document.getElementById('ring');
ringEl.style.strokeDasharray=CIRC;
ringEl.style.strokeDashoffset=0;

let actx=null,asrc=null,again=null,cvol=.5;
const apl=document.getElementById('apl');

// ═══════════════════════════════════════════════════════════════
//  TAB SWITCHING
// ═══════════════════════════════════════════════════════════════
function stab(t,el){
  document.querySelectorAll('.pnl').forEach(p=>p.classList.remove('on'));
  document.querySelectorAll('.tab').forEach(b=>b.classList.remove('on'));
  document.getElementById('p-'+t).classList.add('on');
  el.classList.add('on');
  if(t==='stats')lstats();
  if(t==='hist')lhist();
}

function satab(t,el){
  document.querySelectorAll('.apnl').forEach(p=>p.classList.remove('on'));
  document.querySelectorAll('.atab').forEach(b=>b.classList.remove('on'));
  document.getElementById('a-'+t).classList.add('on');
  el.classList.add('on');
  if(t==='upl')lupl();
}

// ═══════════════════════════════════════════════════════════════
//  DURATION SELECTION
// ═══════════════════════════════════════════════════════════════
function sd(m,el){
  if(run)return;
  m=parseInt(m)||25;
  dur=m*60;tl=dur;
  document.getElementById('cmin').value=m;
  utd();uring();
  if(el&&el.classList){
    document.querySelectorAll('.dur button').forEach(b=>b.classList.remove('on'));
    el.classList.add('on');
  }
}

// ═══════════════════════════════════════════════════════════════
//  DISPLAY UPDATES
// ═══════════════════════════════════════════════════════════════
function utd(){
  const m=Math.floor(tl/60),s=tl%60;
  document.getElementById('tdisp').textContent=
    String(m).padStart(2,'0')+':'+String(s).padStart(2,'0');
}

function uring(){
  const p=1-(tl/dur);
  ringEl.style.strokeDashoffset=CIRC*(1-p);
}

// ═══════════════════════════════════════════════════════════════
//  TIMER TOGGLE / RESET
// ═══════════════════════════════════════════════════════════════
async function toggle(){
  if(!run){
    run=true;elapsed=0;dists=[];
    document.getElementById('sbtn').innerHTML='<span class="mi">pause</span> Pause';
    document.getElementById('rbtn').style.display='';
    document.getElementById('dcrd').style.display='';
    document.getElementById('tlbl').textContent='Focusing...';
    document.getElementById('dlist').innerHTML='';
    document.getElementById('timerRing').classList.add('running');

    try{
      const r=await fetch('/api/session/start',{method:'POST',
        headers:{'Content-Type':'application/json'},
        body:JSON.stringify({duration_minutes:dur/60})});
      const d=await r.json();sid=d.session_id;
    }catch(e){}

    iv=setInterval(()=>{
      tl--;elapsed++;utd();uring();
      if(elapsed%8===0)dtree(tfm+elapsed/60);
      if(tl<=0)done(true);
    },1000);
  }else{
    clearInterval(iv);run=false;
    document.getElementById('sbtn').innerHTML='<span class="mi">play_arrow</span> Resume';
    document.getElementById('tlbl').textContent='Paused';
    document.getElementById('timerRing').classList.remove('running');
  }
}

async function done(ok){
  clearInterval(iv);run=false;
  document.getElementById('timerRing').classList.remove('running');
  const am=elapsed/60;
  if(sid){
    try{await fetch('/api/session/end',{method:'POST',
      headers:{'Content-Type':'application/json'},
      body:JSON.stringify({session_id:sid,actual_minutes:am,completed:ok})});
    }catch(e){}
  }
  tfm+=am;
  document.getElementById('tfdisp').textContent=Math.round(tfm);
  ulvl();dtree(tfm);
  if(ok){
    toast('check_circle','Session completed! Your tree grew!','ok');
    document.getElementById('tlbl').textContent='Completed!';
  }
  document.getElementById('sbtn').innerHTML='<span class="mi">play_arrow</span> Start';
  sid=null;
}

function rst(){
  if(run&&sid)done(false);
  clearInterval(iv);run=false;tl=dur;elapsed=0;utd();uring();
  document.getElementById('sbtn').innerHTML='<span class="mi">play_arrow</span> Start';
  document.getElementById('rbtn').style.display='none';
  document.getElementById('dcrd').style.display='none';
  document.getElementById('tlbl').textContent='Ready to focus';
  document.getElementById('timerRing').classList.remove('running');
}

// ═══════════════════════════════════════════════════════════════
//  DISTRACTIONS
// ═══════════════════════════════════════════════════════════════
async function ld(){
  const inp=document.getElementById('dnote');
  const n=inp.value.trim();if(!n)return;
  const t=new Date().toLocaleTimeString();
  dists.push({t,n});
  if(sid){try{await fetch('/api/session/distraction',{method:'POST',
    headers:{'Content-Type':'application/json'},
    body:JSON.stringify({session_id:sid,note:n})});}catch(e){}}
  document.getElementById('dlist').innerHTML=dists.map(d=>
    '<div class="dit"><span class="dn"><span class="mi mi-sm">warning</span> '+d.n+
    '</span><span class="dt">'+d.t+'</span></div>').join('');
  inp.value='';toast('warning','Distraction logged','err');
}

// ═══════════════════════════════════════════════════════════════
//  AUDIO ENGINE
// ═══════════════════════════════════════════════════════════════
function gctx(){
  if(!actx){
    actx=new(window.AudioContext||window.webkitAudioContext)();
    again=actx.createGain();again.gain.value=cvol;
    again.connect(actx.destination);
  }
  return actx;
}

function gnoise(type){
  const c=gctx(),sr=c.sampleRate,len=sr*4,buf=c.createBuffer(2,len,sr);
  for(let ch=0;ch<2;ch++){
    const d=buf.getChannelData(ch);let last=0;
    if(type==='rain'||type==='ocean'){
      for(let i=0;i<len;i++){
        let w=Math.random()*2-1;d[i]=(last+.02*w)/1.02;last=d[i];d[i]*=3.5;
        if(type==='ocean')d[i]*=.5+.5*Math.sin(2*Math.PI*i/(sr*3));
      }
    }else if(type==='forest'||type==='birds'){
      for(let i=0;i<len;i++){
        let w=Math.random()*2-1;d[i]=(last+.01*w)/1.01;last=d[i];d[i]*=2;
        if(Math.random()<.0001)
          for(let j=0;j<Math.min(2000,len-i);j++)
            d[i+j]+=.3*Math.sin(2*Math.PI*(2000+Math.random()*3000)*j/sr)*Math.exp(-j/500);
      }
    }else if(type==='fire'){
      for(let i=0;i<len;i++){
        let w=Math.random()*2-1;d[i]=(last+.04*w)/1.04;last=d[i];d[i]*=4;
        if(Math.random()<.001)d[i]+=(Math.random()-.5)*.8;
      }
    }else if(type==='wind'){
      for(let i=0;i<len;i++){
        let w=Math.random()*2-1;d[i]=(last+.005*w)/1.005;last=d[i];
        d[i]*=5;d[i]*=.3+.7*Math.sin(2*Math.PI*i/(sr*8));
      }
    }else if(type==='cafe'){
      let b0=0,b1=0,b2=0,b3=0,b4=0,b5=0,b6=0;
      for(let i=0;i<len;i++){
        let w=Math.random()*2-1;
        b0=.99886*b0+w*.0555179;b1=.99332*b1+w*.0750759;
        b2=.969*b2+w*.153852;b3=.8665*b3+w*.3104856;
        b4=.55*b4+w*.5329522;b5=-.7616*b5-w*.016898;
        d[i]=(b0+b1+b2+b3+b4+b5+b6+w*.5362)*.11;b6=w*.115926;
      }
    }
  }
  return buf;
}

function pamb(type,el){
  samb();const c=gctx();
  asrc=c.createBufferSource();asrc.buffer=gnoise(type);
  asrc.loop=true;asrc.connect(again);asrc.start();
  document.querySelectorAll('.sb').forEach(b=>b.classList.remove('on'));
  if(el)el.classList.add('on');
  toast('music_note','Playing '+type+' sounds','nfo');
}

function samb(){
  if(asrc){try{asrc.stop();}catch(e){} asrc=null;}
  apl.pause();apl.currentTime=0;
  document.querySelectorAll('.sb').forEach(b=>b.classList.remove('on'));
}

function plnk(){
  const u=document.getElementById('aurl').value.trim();
  if(!u)return toast('error','Enter a URL','err');
  samb();apl.src=u;apl.volume=cvol;
  apl.play().catch(()=>toast('error','Cannot play URL','err'));
  toast('link','Playing from link','nfo');
}

async function uaud(){
  const f=document.getElementById('afile').files[0];if(!f)return;
  const fd=new FormData();fd.append('file',f);
  try{
    const r=await fetch('/api/sound/upload',{method:'POST',body:fd});
    const d=await r.json();
    toast('upload_file','Uploaded: '+d.name,'ok');lupl();
  }catch(e){toast('error','Upload failed','err');}
}

async function lupl(){
  try{
    const r=await fetch('/api/sound/list');const s=await r.json();
    document.getElementById('ulist').innerHTML=s.map(x=>
      '<button class="sb" onclick="pupl(\''+x.id+'\')" style="margin:3px">'+
      '<span class="si">🎵</span>'+x.name+'</button>').join('');
  }catch(e){}
}

function pupl(id){
  samb();apl.src='/api/sound/'+id;apl.volume=cvol;apl.play();
  toast('music_note','Playing uploaded sound','nfo');
}

function svol(v){
  cvol=v/100;document.getElementById('vv').textContent=v+'%';
  if(again)again.gain.value=cvol;apl.volume=cvol;
  const ico=document.getElementById('volIcon');
  if(v==0)ico.textContent='volume_off';
  else if(v<50)ico.textContent='volume_down';
  else ico.textContent='volume_up';
}

// ═══════════════════════════════════════════════════════════════
//  TREE RENDERING (Canvas)
// ═══════════════════════════════════════════════════════════════
function dtree(fm){
  const cv=document.getElementById('treeCanvas'),c=cv.getContext('2d'),
    W=cv.width,H=cv.height;
  c.clearRect(0,0,W,H);

  // Sky gradient
  let g=c.createLinearGradient(0,0,0,H);
  g.addColorStop(0,'#080d18');g.addColorStop(.4,'#0e1a30');
  g.addColorStop(.7,'#142235');g.addColorStop(1,'#162e28');
  c.fillStyle=g;c.fillRect(0,0,W,H);

  // Moon
  c.fillStyle='rgba(255,255,240,0.9)';
  c.beginPath();c.arc(W-60,50,20,0,Math.PI*2);c.fill();
  c.fillStyle='rgba(255,255,240,0.04)';
  c.beginPath();c.arc(W-60,50,40,0,Math.PI*2);c.fill();

  // Stars
  for(let i=0;i<45;i++){
    let x=((Math.sin(i*127.1+311.7)*.5+.5)*W);
    let y=((Math.sin(i*269.5+183.3)*.5+.5)*H*.45);
    let r=Math.random()*1.8+.4;
    let a=.2+Math.random()*.5;
    c.fillStyle='rgba(255,255,255,'+a+')';
    c.beginPath();c.arc(x,y,r,0,Math.PI*2);c.fill();
  }

  // Ground with gradient
  let gg=c.createLinearGradient(0,H-40,0,H);
  gg.addColorStop(0,'#162e28');gg.addColorStop(1,'#1a3a30');
  c.fillStyle=gg;
  c.beginPath();c.ellipse(W/2,H-6,W/2+30,32,0,0,Math.PI*2);c.fill();

  // Grass blades
  for(let i=0;i<70;i++){
    let x=(i/70)*W;let h=4+Math.random()*16;
    c.strokeStyle='rgba(74,222,128,'+(0.12+Math.random()*.22)+')';
    c.lineWidth=1;c.beginPath();c.moveTo(x,H-8);
    c.quadraticCurveTo(x+Math.random()*8-4,H-8-h/2,x+Math.random()*6-3,H-8-h);
    c.stroke();
  }

  const gr=Math.min(fm/300,1),bx=W/2,by=H-24;

  // SEED stage
  if(fm<1){
    c.fillStyle='#8B6914';c.beginPath();
    c.ellipse(bx,by-4,8,5,0,0,Math.PI*2);c.fill();
    c.fillStyle='#A0782C';c.beginPath();
    c.ellipse(bx,by-5,6,3,.1,0,Math.PI);c.fill();
    // Sparkle
    c.fillStyle='rgba(251,191,36,0.6)';
    c.beginPath();c.arc(bx+6,by-10,2,0,Math.PI*2);c.fill();
    return;
  }

  // SPROUT stage
  if(fm<10){
    let sh=10+(fm/10)*35;
    c.strokeStyle='#4ade80';c.lineWidth=3;c.lineCap='round';
    c.beginPath();c.moveTo(bx,by);
    c.bezierCurveTo(bx+3,by-sh*.3,bx-2,by-sh*.7,bx,by-sh);c.stroke();
    dlf(c,bx-3,by-sh,13,-.6,'#4ade80');
    dlf(c,bx+3,by-sh+6,11,.6,'#22c55e');
    // Tiny glow
    c.fillStyle='rgba(74,222,128,0.08)';
    c.beginPath();c.arc(bx,by-sh,20,0,Math.PI*2);c.fill();
    return;
  }

  // FULL TREE
  const th=45+gr*115,tw=4+gr*10;

  // Trunk
  c.strokeStyle='#6B4E1B';c.lineWidth=tw;c.lineCap='round';
  c.beginPath();c.moveTo(bx,by);
  c.bezierCurveTo(bx-2,by-th*.3,bx+2,by-th*.7,bx,by-th);
  c.stroke();

  // Trunk highlight
  c.strokeStyle='rgba(160,120,44,0.25)';c.lineWidth=tw*.3;
  c.beginPath();c.moveTo(bx-tw*.15,by);
  c.bezierCurveTo(bx-tw*.15-1,by-th*.3,bx-tw*.15+1,by-th*.7,bx-tw*.15,by-th);
  c.stroke();

  // Bark marks
  for(let i=0;i<th;i+=8){
    c.strokeStyle='rgba(90,60,20,'+(0.1+Math.random()*.15)+')';
    c.lineWidth=1;c.beginPath();
    c.moveTo(bx-tw*.35+2,by-i);
    c.lineTo(bx-tw*.35+5+Math.random()*4,by-i-4);c.stroke();
  }

  // Roots
  for(let i=0;i<3;i++){
    const ra=(-0.3+i*0.3)+Math.random()*.2;
    const rl=15+Math.random()*20;
    c.strokeStyle='rgba(107,78,27,0.6)';c.lineWidth=2+Math.random()*2;
    c.beginPath();c.moveTo(bx,by);
    c.quadraticCurveTo(bx+Math.cos(ra)*rl*.5,by+8,bx+Math.cos(ra)*rl,by+5);
    c.stroke();
  }

  // Branches (recursive fractal)
  const md=Math.floor(2+gr*5);
  dbr(c,bx,by-th,th*.48,-Math.PI/2,md,gr,tw*.55);

  // Canopy glow
  let gl=c.createRadialGradient(bx,by-th-10,5,bx,by-th-10,th*.75);
  gl.addColorStop(0,'rgba(74,222,128,.07)');gl.addColorStop(1,'transparent');
  c.fillStyle=gl;c.fillRect(0,0,W,H);

  // Fireflies for mature trees
  if(gr>.4){
    const nf=Math.floor(gr*8);
    for(let i=0;i<nf;i++){
      const fx=bx+(Math.random()-.5)*120;
      const fy=by-th-40+Math.random()*80;
      const fa=.3+Math.random()*.5;
      c.fillStyle='rgba(251,191,36,'+fa+')';
      c.beginPath();c.arc(fx,fy,1.5,0,Math.PI*2);c.fill();
      c.fillStyle='rgba(251,191,36,'+(fa*.2)+')';
      c.beginPath();c.arc(fx,fy,5,0,Math.PI*2);c.fill();
    }
  }
}

function dbr(c,x,y,len,ang,dep,gr,w){
  if(dep<=0||len<4)return;
  const ex=x+Math.cos(ang)*len,ey=y+Math.sin(ang)*len;

  // Branch
  const brColor=dep>3?'#6B4E1B':'rgba(74,180,90,0.8)';
  c.strokeStyle=brColor;c.lineWidth=Math.max(1,w);c.lineCap='round';
  c.beginPath();c.moveTo(x,y);
  const mx=(x+ex)/2+(Math.random()-.5)*4;
  const my=(y+ey)/2+(Math.random()-.5)*4;
  c.quadraticCurveTo(mx,my,ex,ey);c.stroke();

  // Leaves at branch tips
  if(dep<=2){
    const ls=4+gr*16;
    const cols=['rgba(74,222,128,','rgba(34,197,94,','rgba(22,163,74,',
      'rgba(134,239,172,','rgba(163,230,53,'];
    const nc=3+Math.floor(gr*6);
    for(let i=0;i<nc;i++){
      const lx=ex+(Math.random()-.5)*ls*2.2;
      const ly=ey+(Math.random()-.5)*ls*2.2;
      const lr=2.5+Math.random()*ls*.5;
      const col=cols[Math.floor(Math.random()*cols.length)];
      const a=.4+Math.random()*.6;
      c.fillStyle=col+a+')';
      c.beginPath();c.arc(lx,ly,lr,0,Math.PI*2);c.fill();
    }
    // Fruits / flowers
    if(gr>.5&&Math.random()<.35){
      const fx=ex+(Math.random()-.5)*10;
      const fy=ey+Math.random()*5;
      const fc=Math.random()>.5?'rgba(248,113,113,0.9)':'rgba(251,191,36,0.9)';
      c.fillStyle=fc;c.beginPath();c.arc(fx,fy,2.5+gr,0,Math.PI*2);c.fill();
      c.fillStyle=fc.replace('0.9','0.2');
      c.beginPath();c.arc(fx,fy,5+gr*2,0,Math.PI*2);c.fill();
    }
    // Cherry blossoms for grand trees
    if(gr>.7&&Math.random()<.2){
      c.fillStyle='rgba(244,114,182,0.7)';
      for(let p=0;p<3;p++){
        c.beginPath();
        c.arc(ex+(Math.random()-.5)*20,ey+(Math.random()-.5)*20,
          2+Math.random()*2,0,Math.PI*2);c.fill();
      }
    }
  }

  const ba=.35+(1-gr)*.2;const lf=.62+Math.random()*.12;
  dbr(c,ex,ey,len*lf,ang-ba,dep-1,gr,w*.6);
  dbr(c,ex,ey,len*lf,ang+ba,dep-1,gr,w*.6);
  if(dep>3&&Math.random()<.5)
    dbr(c,ex,ey,len*.4,ang+(Math.random()-.5)*.9,dep-2,gr,w*.35);
}

function dlf(c,x,y,sz,a,col){
  c.save();c.translate(x,y);c.rotate(a);c.fillStyle=col;
  c.beginPath();c.moveTo(0,0);
  c.quadraticCurveTo(sz/2,-sz/2,sz,0);
  c.quadraticCurveTo(sz/2,sz/2,0,0);c.fill();
  // Leaf vein
  c.strokeStyle='rgba(0,0,0,0.15)';c.lineWidth=.5;
  c.beginPath();c.moveTo(1,0);c.lineTo(sz-2,0);c.stroke();
  c.restore();
}

function ulvl(){
  const lvs=[
    [0,'Seed 🌰'],[5,'Sprout 🌱'],[20,'Seedling 🌿'],
    [45,'Sapling 🪴'],[90,'Young Tree 🌳'],[150,'Mature Tree 🌲'],
    [240,'Grand Tree 🌴'],[400,'Ancient Tree 🏔'],
    [700,'Legendary Tree ✨'],[1000,'World Tree 🌍']
  ];
  let lv=lvs[0][1];
  for(const[m,n]of lvs)if(tfm>=m)lv=n;
  document.getElementById('tlvl').textContent=lv;
}

// ═══════════════════════════════════════════════════════════════
//  ANALYTICS
// ═══════════════════════════════════════════════════════════════
async function lstats(){
  try{
    const r=await fetch('/api/analytics'),d=await r.json();
    document.getElementById('sgrid').innerHTML=
      '<div class="sc"><span class="mi ico">schedule</span><div class="v">'+
        Math.round(d.total_minutes)+'</div><div class="l">Minutes This Week</div></div>'+
      '<div class="sc"><span class="mi ico">target</span><div class="v">'+
        d.total_sessions+'</div><div class="l">Sessions</div></div>'+
      '<div class="sc"><span class="mi ico">avg_pace</span><div class="v">'+
        Math.round(d.avg_session_length)+'</div><div class="l">Avg Length (min)</div></div>'+
      '<div class="sc"><span class="mi ico">warning</span><div class="v">'+
        d.total_distractions+'</div><div class="l">Distractions</div></div>'+
      '<div class="sc"><span class="mi ico">emoji_events</span><div class="v">'+
        Math.round(d.total_focus_all_time)+'</div><div class="l">All-Time Minutes</div></div>'+
      '<div class="sc"><span class="mi ico">verified</span><div class="v">'+
        Math.round(d.completion_rate)+'%</div><div class="l">Completion Rate</div></div>'+
      '<div class="sc"><span class="mi ico">bolt</span><div class="v">'+
        Math.round(d.longest_session)+'</div><div class="l">Longest Session</div></div>'+
      '<div class="sc"><span class="mi ico">local_fire_department</span><div class="v">'+
        d.streak_days+'</div><div class="l">Streak Days</div></div>';

    const mm=Math.max(...d.daily_minutes,1);
    const ms=Math.max(...d.daily_sessions,1);
    const md2=Math.max(...d.daily_distractions,1);

    document.getElementById('ch1').innerHTML=d.day_labels.map((l,i)=>{
      const h=(d.daily_minutes[i]/mm)*170;
      return '<div class="bgrp"><div class="bv" style="color:var(--g1)">'+
        Math.round(d.daily_minutes[i])+'</div><div class="bar g1" style="height:'+
        Math.max(4,h)+'px"></div><div class="bl">'+l+'</div></div>';
    }).join('');

    document.getElementById('ch2').innerHTML=d.day_labels.map((l,i)=>{
      const h=(d.daily_sessions[i]/ms)*170;
      return '<div class="bgrp"><div class="bv" style="color:var(--g3)">'+
        d.daily_sessions[i]+'</div><div class="bar g2" style="height:'+
        Math.max(4,h)+'px"></div><div class="bl">'+l+'</div></div>';
    }).join('');

    document.getElementById('ch3').innerHTML=d.day_labels.map((l,i)=>{
      const h=(d.daily_distractions[i]/md2)*170;
      return '<div class="bgrp"><div class="bv" style="color:var(--rd)">'+
        d.daily_distractions[i]+'</div><div class="bar g3" style="height:'+
        Math.max(4,h)+'px"></div><div class="bl">'+l+'</div></div>';
    }).join('');
  }catch(e){console.error(e);}
}

// ═══════════════════════════════════════════════════════════════
//  SESSION HISTORY
// ═══════════════════════════════════════════════════════════════
async function lhist(){
  try{
    const r=await fetch('/api/sessions'),s=await r.json();
    if(s.length===0){
      document.getElementById('slist').innerHTML=
        '<div class="empty"><span class="mi">eco</span>'+
        '<p>No sessions yet. Start focusing!</p></div>';
      return;
    }
    document.getElementById('slist').innerHTML=s.slice().reverse().map(x=>
      '<div class="si2"><div class="sinf">'+
      '<div class="sd"><span class="mi mi-sm">calendar_today</span>'+x.date+'</div>'+
      '<div class="st2"><span class="mi mi-sm">timer</span> '+
        Math.round(x.actual_minutes)+' min &mdash; '+
        x.start_time+(x.end_time?' &rarr; '+x.end_time:'')+'</div>'+
      '<div class="sm2"><span class="mi">warning</span>'+
        x.distractions.length+' distraction'+(x.distractions.length!==1?'s':'')+'</div>'+
      '</div><span class="ss '+(x.completed?'c':'ic')+'">'+
      '<span class="mi">'+(x.completed?'check_circle':'cancel')+'</span>'+
      (x.completed?'Done':'Stopped')+'</span></div>'
    ).join('');
  }catch(e){console.error(e);}
}

// ═══════════════════════════════════════════════════════════════
//  TOAST NOTIFICATIONS
// ═══════════════════════════════════════════════════════════════
function toast(icon,msg,type){
  const t=document.createElement('div');
  t.className='toast '+type;
  t.innerHTML='<span class="mi">'+icon+'</span>'+msg;
  document.body.appendChild(t);
  setTimeout(()=>t.remove(),3200);
}

// ═══════════════════════════════════════════════════════════════
//  INITIALIZATION
// ═══════════════════════════════════════════════════════════════
async function init(){
  try{
    const r=await fetch('/api/total');
    const d=await r.json();tfm=d.total||0;
  }catch(e){}
  document.getElementById('tfdisp').textContent=Math.round(tfm);
  ulvl();utd();dtree(tfm);
}
init();
</script>
</body>
</html>"####.to_string()
}