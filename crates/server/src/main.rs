//! Server-Skelett (Phase 0).
//!
//! Server-autoritatives Modell (DESIGN.md §5.3): der Server hält den Welt-Zustand,
//! Clients stellen nur dar. Dieses Skelett bietet:
//!
//! - `GET /health` — Lebenszeichen.
//! - `GET /system` — der aktuelle Systemzustand als JSON (aus `core`).
//! - `GET /ws` — WebSocket, der die Weltzeit tickt und Körperpositionen streamt.
//!
//! Persistenz (Postgres) und echte Befehlsverarbeitung folgen in späteren Phasen.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use gamecore::{SimClock, System};
use serde::Serialize;
use tokio::sync::Mutex;

/// Geteilter Server-Zustand. In Phase 0 nur im Speicher; später aus Postgres
/// geladen und dorthin persistiert.
struct AppState {
    system: System,
    clock: Mutex<SimClock>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let state = Arc::new(AppState {
        system: gamecore::demo_home_system(),
        clock: Mutex::new(SimClock::default()),
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/system", get(system))
        .route("/ws", get(ws_upgrade))
        .with_state(state);

    let addr = "127.0.0.1:8080";
    tracing::info!("Server lauscht auf http://{addr}");
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
async fn system(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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

    loop {
        interval.tick().await;

        let t = {
            let mut clock = state.clock.lock().await;
            clock.advance(TIME_SCALE);
            clock.now()
        };

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
