use sqlx::{AnyPool, PgPool};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub calc_version: String,
}

/// SQLite（ローカル）と Postgres（Shuttle）の両方に対応するプール。
#[derive(Clone)]
pub enum DbPool {
    Any(AnyPool),
    Pg(PgPool),
}

pub const DEFAULT_DB_URL: &str = "sqlite://./mj.sqlite?mode=rwc";
