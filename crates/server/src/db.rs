//! Postgres-Anbindung (Phase-0-Grundlage).
//!
//! Bewusst **optional**: ist `DATABASE_URL` gesetzt und die DB erreichbar, läuft
//! der Server persistent (lädt die Welt bzw. legt sie an und sichert die
//! Sim-Zeit). Fehlt sie, läuft das Skelett wie bisher rein im Speicher — damit
//! bleibt das Repo ohne lokale DB lauffähig.

use std::time::Duration;

use gamecore::System;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

/// Stellt eine Verbindung her, sofern `DATABASE_URL` gesetzt ist, und wendet die
/// Migrationen an. `Ok(None)` bedeutet „keine DB konfiguriert" (kein Fehler).
///
/// Bei nicht erreichbarer DB schlägt es nach einem **kurzen** Timeout fehl,
/// statt minutenlang zu hängen — der Aufrufer fällt dann auf den Speicherbetrieb
/// zurück, und der Server startet trotzdem zügig.
pub async fn connect_and_migrate() -> Result<Option<PgPool>, sqlx::Error> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) if !u.trim().is_empty() => u,
        _ => return Ok(None),
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        // Schnell scheitern, wenn Postgres nicht läuft (Default wäre 30 s).
        .acquire_timeout(Duration::from_secs(4))
        .connect(&url)
        .await?;

    // Migrationen aus crates/server/migrations werden zur Compile-Zeit eingebettet.
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(Some(pool))
}

/// Lädt die persistierte Welt oder legt sie aus `default` an (Singleton, id = 1).
/// Gibt `(sim_time, system)` zurück.
pub async fn load_or_seed_world(
    pool: &PgPool,
    default: &System,
) -> Result<(f64, System), sqlx::Error> {
    if let Some(row) = sqlx::query("SELECT sim_time, system FROM world WHERE id = 1")
        .fetch_optional(pool)
        .await?
    {
        let sim_time: f64 = row.get("sim_time");
        let system: sqlx::types::Json<System> = row.get("system");
        Ok((sim_time, system.0))
    } else {
        sqlx::query("INSERT INTO world (id, sim_time, system) VALUES (1, 0, $1)")
            .bind(sqlx::types::Json(default))
            .execute(pool)
            .await?;
        Ok((0.0, default.clone()))
    }
}

/// Sichert die fortgeschrittene Sim-Zeit (Welt-Geometrie ist „on rails", daher
/// genügt der Zeitstempel, um die Welt nach einem Neustart wieder einzuholen).
pub async fn save_sim_time(pool: &PgPool, sim_time: f64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE world SET sim_time = $1, updated_at = now() WHERE id = 1")
        .bind(sim_time)
        .execute(pool)
        .await?;
    Ok(())
}
