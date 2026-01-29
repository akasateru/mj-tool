use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use mj_domain::{calc_score, validate_fact, CalcResult, Fact, RULESET_V1};
use serde::{Deserialize, Serialize};
use sqlx::{any::AnyPoolOptions, AnyPool};
use sqlx::Row;
use time::OffsetDateTime;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use std::time::Duration;

#[derive(Clone)]
struct AppState {
    db: AnyPool,
    calc_version: String,
}

const DEFAULT_DB_URL: &str = "sqlite://./mj.sqlite?mode=rwc";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "mj_api=debug,tower_http=info,axum=info,sqlx=warn".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // sqlx::AnyPool を使う場合は driver 登録が必要
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

async fn init_db(db: &AnyPool) -> anyhow::Result<()> {
    // MVP用（SQLite/Postgres両対応の“最小”スキーマ）。
    // Supabase本番スキーマ（RLS/uuid/jsonb等）は supabase/migrations を参照。
    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS hands (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  rule_set_id TEXT NOT NULL,
  calc_version TEXT NOT NULL,
  fact_json TEXT NOT NULL,
  result_json TEXT NOT NULL,
  memo TEXT,
  tags_json TEXT
);
CREATE INDEX IF NOT EXISTS idx_hands_user_created_at ON hands(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_hands_user_memo ON hands(user_id, memo);
"#,
    )
    .execute(db)
    .await?;
    Ok(())
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "ok": true }))
}

#[derive(Debug, Deserialize)]
struct CalcRequest {
    fact: Fact,
}

#[derive(Debug, Serialize)]
struct CalcResponse {
    calc_version: String,
    result: CalcResult,
}

async fn calc(State(state): State<AppState>, Json(req): Json<CalcRequest>) -> axum::response::Response {
    match validate_fact(req.fact) {
        Ok(valid) => {
            let result = calc_score(&valid, RULESET_V1);
            (
                StatusCode::OK,
                Json(CalcResponse {
                    calc_version: state.calc_version,
                    result,
                }),
            )
                .into_response()
        }
        Err(errors) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "errors": errors })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct CreateHandRequest {
    fact: Fact,
    memo: Option<String>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct HandRecord {
    id: Uuid,
    user_id: String,
    created_at: OffsetDateTime,
    rule_set_id: String,
    calc_version: String,
    fact: Fact,
    result: CalcResult,
    memo: Option<String>,
    tags: Vec<String>,
}

fn user_id_from_headers(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "local".to_string())
}

async fn create_hand(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateHandRequest>,
) -> impl IntoResponse {
    let user_id = user_id_from_headers(&headers);

    let valid = match validate_fact(req.fact.clone()) {
        Ok(v) => v,
        Err(errors) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "errors": errors })),
            )
                .into_response();
        }
    };

    let result = calc_score(&valid, RULESET_V1);
    let id = Uuid::new_v4();
    let created_at = OffsetDateTime::now_utc();
    let tags = req.tags.unwrap_or_default();

    let fact_json = serde_json::to_string(&req.fact).unwrap();
    let result_json = serde_json::to_string(&result).unwrap();
    let tags_json = serde_json::to_string(&tags).unwrap();

    if let Err(e) = sqlx::query(
        r#"
INSERT INTO hands (id, user_id, created_at, rule_set_id, calc_version, fact_json, result_json, memo, tags_json)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
    )
    .bind(id.to_string())
    .bind(&user_id)
    .bind(created_at.format(&time::format_description::well_known::Rfc3339).unwrap())
    .bind(&result.rule_set_id)
    .bind(&state.calc_version)
    .bind(fact_json)
    .bind(result_json)
    .bind(req.memo.clone())
    .bind(tags_json)
    .execute(&state.db)
    .await
    {
        tracing::error!(error = ?e, "failed to insert hand");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "db_error" })))
            .into_response();
    }

    (
        StatusCode::CREATED,
        Json(HandRecord {
            id,
            user_id,
            created_at,
            rule_set_id: result.rule_set_id.clone(),
            calc_version: state.calc_version,
            fact: req.fact,
            result,
            memo: req.memo,
            tags,
        }),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
struct ListHandsQuery {
    q: Option<String>,
    limit: Option<u32>,
}

#[derive(Debug, Serialize)]
struct HandSummary {
    id: Uuid,
    created_at: OffsetDateTime,
    display_hand_points: String,
    total_to_winner: i32,
    memo: Option<String>,
    tags: Vec<String>,
}

async fn list_hands(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(q): Query<ListHandsQuery>,
) -> impl IntoResponse {
    let user_id = user_id_from_headers(&headers);
    let limit = q.limit.unwrap_or(50).min(200) as i64;
    let keyword = q.q.unwrap_or_default();

    let mut query = String::from(
        r#"
SELECT id, created_at, result_json, memo, tags_json
FROM hands
WHERE user_id = ?
"#,
    );
    let mut binds: Vec<String> = vec![];
    if !keyword.is_empty() {
        query.push_str("  AND (memo LIKE ? OR tags_json LIKE ?)\n");
        let like = format!("%{}%", keyword);
        binds.push(like.clone());
        binds.push(like);
    }
    query.push_str("ORDER BY created_at DESC\nLIMIT ?\n");

    let mut qx = sqlx::query(&query).bind(&user_id);
    for b in binds {
        qx = qx.bind(b);
    }
    let rows = match qx.bind(limit).fetch_all(&state.db).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = ?e, "failed to list hands");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "db_error" })),
            )
                .into_response();
        }
    };

    let summaries = rows
        .into_iter()
        .filter_map(|row| {
            let id: String = row.try_get("id").ok()?;
            let created_at: String = row.try_get("created_at").ok()?;
            let result_json: String = row.try_get("result_json").ok()?;
            let memo: Option<String> = row.try_get("memo").ok().flatten();
            let tags_json: Option<String> = row.try_get("tags_json").ok().flatten();

            let id = Uuid::parse_str(&id).ok()?;
            let created_at = OffsetDateTime::parse(
                &created_at,
                &time::format_description::well_known::Rfc3339,
            )
            .ok()?;
            let result: CalcResult = serde_json::from_str(&result_json).ok()?;
            let tags: Vec<String> = tags_json
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();

            Some(HandSummary {
                id,
                created_at,
                display_hand_points: result.display_hand_points.clone(),
                total_to_winner: result.totals.total_to_winner,
                memo,
                tags,
            })
        })
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(summaries)).into_response()
}

async fn get_hand(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user_id = user_id_from_headers(&headers);

    let row = match sqlx::query(
        r#"
SELECT id, user_id, created_at, rule_set_id, calc_version, fact_json, result_json, memo, tags_json
FROM hands
WHERE user_id = ?
  AND id = ?
LIMIT 1
"#,
    )
    .bind(&user_id)
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = ?e, "failed to get hand");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "db_error" })),
            )
                .into_response();
        }
    };

    let Some(row) = row else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "not_found" }))).into_response();
    };

    let id: String = row.try_get("id").unwrap();
    let row_user_id: String = row.try_get("user_id").unwrap();
    let created_at: String = row.try_get("created_at").unwrap();
    let rule_set_id: String = row.try_get("rule_set_id").unwrap();
    let calc_version: String = row.try_get("calc_version").unwrap();
    let fact_json: String = row.try_get("fact_json").unwrap();
    let result_json: String = row.try_get("result_json").unwrap();
    let memo: Option<String> = row.try_get("memo").ok().flatten();
    let tags_json: Option<String> = row.try_get("tags_json").ok().flatten();

    let id = Uuid::parse_str(&id).unwrap();
    let created_at = OffsetDateTime::parse(
        &created_at,
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    let fact: Fact = serde_json::from_str(&fact_json).unwrap();
    let result: CalcResult = serde_json::from_str(&result_json).unwrap();
    let tags: Vec<String> = tags_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();

    (
        StatusCode::OK,
        Json(HandRecord {
            id,
            user_id: row_user_id,
            created_at,
            rule_set_id,
            calc_version,
            fact,
            result,
            memo,
            tags,
        }),
    )
        .into_response()
}

#[derive(Debug, Serialize)]
struct RecalcResponse {
    stored_calc_version: String,
    current_calc_version: String,
    same: bool,
    stored_result: CalcResult,
    current_result: CalcResult,
}

async fn recalc_hand(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user_id = user_id_from_headers(&headers);

    let row = match sqlx::query(
        r#"
SELECT calc_version, fact_json, result_json
FROM hands
WHERE user_id = ?
  AND id = ?
LIMIT 1
"#,
    )
    .bind(&user_id)
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = ?e, "failed to get hand for recalc");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "db_error" })),
            )
                .into_response();
        }
    };

    let Some(row) = row else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "not_found" }))).into_response();
    };

    let stored_calc_version: String = row.try_get("calc_version").unwrap();
    let fact_json: String = row.try_get("fact_json").unwrap();
    let result_json: String = row.try_get("result_json").unwrap();

    let fact: Fact = match serde_json::from_str(&fact_json) {
        Ok(v) => v,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "corrupt_fact" }))).into_response(),
    };
    let stored_result: CalcResult = match serde_json::from_str(&result_json) {
        Ok(v) => v,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "corrupt_result" }))).into_response(),
    };

    let valid = match validate_fact(fact) {
        Ok(v) => v,
        Err(errors) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "errors": errors })),
            )
                .into_response();
        }
    };
    let current_result = calc_score(&valid, RULESET_V1);
    let same = stored_result == current_result && stored_calc_version == state.calc_version;

    (
        StatusCode::OK,
        Json(RecalcResponse {
            stored_calc_version,
            current_calc_version: state.calc_version,
            same,
            stored_result,
            current_result,
        }),
    )
        .into_response()
}
