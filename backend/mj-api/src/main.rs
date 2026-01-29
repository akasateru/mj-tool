mod db;
mod handlers;
mod models;
mod state;

use axum::routing::{get, post};
use axum::Router;
use sqlx::any::AnyPoolOptions;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use db::init_db;
use handlers::{calc, create_hand, get_hand, health, list_hands, recalc_hand};
use state::{AppState, DEFAULT_DB_URL};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "mj_api=debug,tower_http=info,axum=info,sqlx=warn".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    sqlx::any::install_default_drivers();

    let raw_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DB_URL.to_string());
    let db = {
        tracing::info!(db_url = %raw_db_url, "connecting database");
        let connect_fut = AnyPoolOptions::new()
            .max_connections(5)
            .connect(&raw_db_url);

        match tokio::time::timeout(Duration::from_secs(3), connect_fut).await {
            Ok(Ok(pool)) => pool,
            Ok(Err(sqlx_err)) => {
                let should_fallback = !raw_db_url.starts_with("sqlite");
                if should_fallback {
                    tracing::warn!(error = ?sqlx_err, "db connect failed; fallback to sqlite");
                    AnyPoolOptions::new()
                        .max_connections(5)
                        .connect(DEFAULT_DB_URL)
                        .await?
                } else {
                    return Err(anyhow::anyhow!(sqlx_err));
                }
            }
            Err(elapsed) => {
                let should_fallback = !raw_db_url.starts_with("sqlite");
                if should_fallback {
                    tracing::warn!(error = ?elapsed, "db connect timeout; fallback to sqlite");
                    AnyPoolOptions::new()
                        .max_connections(5)
                        .connect(DEFAULT_DB_URL)
                        .await?
                } else {
                    return Err(anyhow::anyhow!(elapsed));
                }
            }
        }
    };
    init_db(&db).await?;

    let state = AppState {
        db,
        calc_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/calc", post(calc))
        .route("/hands", post(create_hand).get(list_hands))
        .route("/hands/{id}", get(get_hand))
        .route("/hands/{id}/recalc", post(recalc_hand))
        .layer(cors)
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
