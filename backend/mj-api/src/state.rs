use sqlx::AnyPool;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub calc_version: String,
}

/// SQLite（ローカル）と Postgres（Any 経由）の両方に対応するプール。
#[derive(Clone)]
pub struct DbPool(pub AnyPool);

pub const DEFAULT_DB_URL: &str = "sqlite://./mj.sqlite?mode=rwc";
