use sqlx::AnyPool;

#[derive(Clone)]
pub struct AppState {
    pub db: AnyPool,
    pub calc_version: String,
}

pub const DEFAULT_DB_URL: &str = "sqlite://./mj.sqlite?mode=rwc";
