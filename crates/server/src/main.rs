//! Server-Skelett (Phase 0).
//!
//! Server-autoritatives Modell (DESIGN.md §5.3): der Server hält den Welt-Zustand,
//! Clients stellen nur dar. Dieses Skelett bietet:
//!
//! - `GET /health` — Lebenszeichen.
//! - `GET /system` — der aktuelle Systemzustand als JSON (aus `core`).
//! - `GET /ws` — WebSocket, der die Weltzeit tickt und Körperpositionen streamt.
//!
//! Persistenz ist optional über Postgres (siehe [`db`]); ohne `DATABASE_URL`
//! läuft das Skelett rein im Speicher.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use gamecore::{SimClock, System};
use serde::Serialize;
use sqlx::PgPool;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

mod db;

/// Geteilter Server-Zustand. Die Welt-Geometrie ist „on rails", daher genügt
/// der Systemzustand plus die Sim-Zeit; der optionale Pool persistiert beides.
struct AppState {
    system: System,
    clock: Mutex<SimClock>,
    db: Option<PgPool>,
}

#[tokio::main]
async fn main() {
    // `.env` laden (DATABASE_URL etc.), falls vorhanden.
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let default_system = gamecore::demo_home_system();

    // Optionale Persistenz: verbinden, migrieren, Welt laden oder anlegen.
    let (system, sim_time, db) = match db::connect_and_migrate().await {
        Ok(Some(pool)) => match db::load_or_seed_world(&pool, &default_system).await {
            Ok((t, sys)) => {
                tracing::info!("Mit Postgres verbunden — Welt geladen (t = {t:.0}s).");
                (sys, t, Some(pool))
            }
            Err(e) => {
                tracing::error!("Welt konnte nicht geladen werden: {e}. Laufe im Speicher.");
                (default_system, 0.0, None)
            }
        },
        Ok(None) => {
            tracing::warn!("DATABASE_URL nicht gesetzt — laufe ohne Persistenz (im Speicher).");
            (default_system, 0.0, None)
        }
        Err(e) => {
            tracing::warn!(
                "Postgres nicht erreichbar ({e}). Läuft Postgres? In WSL z. B. \
                 `wsl sudo service postgresql start`. Server läuft ohne Persistenz weiter."
            );
            (default_system, 0.0, None)
        }
    };

    let state = Arc::new(AppState {
        system,
        clock: Mutex::new(SimClock::new(sim_time)),
        db,
    });

    // Verzeichnis der gebauten Browser-UI (trunk dist/). Per Env überschreibbar.
    let dist = std::env::var("CLIENT_DIST")
        .unwrap_or_else(|_| "crates/client/dist".to_string());
    let serve_ui = ServeDir::new(&dist).append_index_html_on_directories(true);

    let app = Router::new()
        .route("/health", get(health))
        .route("/system", get(get_system))
        .route("/ws", get(ws_upgrade))
        .with_state(state)
        // Alles Übrige (UI, Wasm, JS) aus dist/ ausliefern; "/" → index.html.
        .fallback_service(serve_ui);

    let addr = "127.0.0.1:8080";
    tracing::info!("Server lauscht auf http://{addr} — UI aus '{dist}'");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Port 8080 konnte nicht gebunden werden");
    axum::serve(listener, app)
        .await
        .expect("Server-Fehler");
}

/// Schlichtes Lebenszeichen.
async fn health() -> &'static str {
    "ok"
}

/// Liefert den aktuellen Systemzustand als JSON.
async fn get_system(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(state.system.clone())
}

/// Ein gestreamtes Positions-Update (DESIGN.md §5.6: nur Zustand, keine Bahnen).
#[derive(Serialize)]
struct Tick {
    /// Weltzeit in Sekunden.
    t: f64,
    /// `(BodyId, x, y)` je Körper.
    positions: Vec<(u32, f64, f64)>,
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| stream_ticks(socket, state))
}

/// Tickt die Weltzeit und streamt Körperpositionen.
///
/// Bewusst langsam (DESIGN.md §2): die Welt ist „on rails", der Client
/// interpoliert zwischen den Updates. Hier ein fester, gemütlicher Takt.
async fn stream_ticks(mut socket: WebSocket, state: Arc<AppState>) {
    // Beschleunigter Zeitfaktor, damit die Bewegung im Skelett sichtbar ist.
    const TIME_SCALE: f64 = 86_400.0; // 1 reale Sekunde ≈ 1 Sim-Tag
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let mut ticks: u64 = 0;

    loop {
        interval.tick().await;

        let t = {
            let mut clock = state.clock.lock().await;
            clock.advance(TIME_SCALE);
            clock.now()
        };

        // Sim-Zeit gelegentlich sichern (nicht jeden Tick) — die Welt ist
        // „on rails", der Zeitstempel reicht, um sie wieder einzuholen.
        ticks += 1;
        if ticks % 10 == 0 {
            if let Some(pool) = &state.db {
                if let Err(e) = db::save_sim_time(pool, t).await {
                    tracing::warn!("Sim-Zeit konnte nicht gesichert werden: {e}");
                }
            }
        }

        let positions = state
            .system
            .bodies
            .iter()
            .filter_map(|b| state.system.position_of(b.id, t).map(|p| (b.id, p.x, p.y)))
            .collect();

        let tick = Tick { t, positions };
        let payload = match serde_json::to_string(&tick) {
            Ok(p) => p,
            Err(_) => continue,
        };

        if socket.send(Message::Text(payload)).await.is_err() {
            // Client ist gegangen.
            break;
        }
    }
}
