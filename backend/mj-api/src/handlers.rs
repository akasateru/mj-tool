use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mj_domain::{
    analyze::{analyze_hand as domain_analyze_hand, AnalyzeHandRequest, AnalyzeHandError},
    calc_score, validate_fact, CalcResult, Fact, RULESET_V1,
};
use sqlx::any::AnyRow;
use sqlx::Row;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::{
    user_id_from_headers, CalcRequest, CalcResponse, CreateHandRequest, HandRecord, HandSummary,
    ListHandsQuery, RecalcResponse,
};
use crate::state::AppState;

fn row_to_hand_summary(row: &AnyRow) -> Option<HandSummary> {
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
}

fn row_to_hand_record(row: &AnyRow) -> Option<HandRecord> {
    let id: String = row.try_get("id").ok()?;
    let user_id: String = row.try_get("user_id").ok()?;
    let created_at: String = row.try_get("created_at").ok()?;
    let rule_set_id: String = row.try_get("rule_set_id").ok()?;
    let calc_version: String = row.try_get("calc_version").ok()?;
    let fact_json: String = row.try_get("fact_json").ok()?;
    let result_json: String = row.try_get("result_json").ok()?;
    let memo: Option<String> = row.try_get("memo").ok().flatten();
    let tags_json: Option<String> = row.try_get("tags_json").ok().flatten();
    let id = Uuid::parse_str(&id).ok()?;
    let created_at = OffsetDateTime::parse(
        &created_at,
        &time::format_description::well_known::Rfc3339,
    )
    .ok()?;
    let fact: Fact = serde_json::from_str(&fact_json).ok()?;
    let result: CalcResult = serde_json::from_str(&result_json).ok()?;
    let tags: Vec<String> = tags_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    Some(HandRecord {
        id,
        user_id,
        created_at,
        rule_set_id,
        calc_version,
        fact,
        result,
        memo,
        tags,
    })
}

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "ok": true }))
}

/// 手牌文字列から翻・符・役を算出する（牌リスト → Fact 候補）。
pub async fn analyze_hand(Json(req): Json<AnalyzeHandRequest>) -> axum::response::Response {
    match domain_analyze_hand(&req) {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(AnalyzeHandError::ParseFailed(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "parse_failed", "detail": msg })),
        )
            .into_response(),
        Err(AnalyzeHandError::NotWinningHand) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({
                "error": "not_winning_hand",
                "detail": "和了形ではありません（14枚の有効な手牌を入力してください）"
            })),
        )
            .into_response(),
    }
}

pub async fn calc(
    State(state): State<AppState>,
    Json(req): Json<CalcRequest>,
) -> axum::response::Response {
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

pub async fn create_hand(
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

    const INSERT_SQL: &str = r#"
INSERT INTO hands (id, user_id, created_at, rule_set_id, calc_version, fact_json, result_json, memo, tags_json)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
"#;
    let exec = sqlx::query(INSERT_SQL)
        .bind(id.to_string())
        .bind(&user_id)
        .bind(created_at.format(&time::format_description::well_known::Rfc3339).unwrap())
        .bind(&result.rule_set_id)
        .bind(&state.calc_version)
        .bind(&fact_json)
        .bind(&result_json)
        .bind(req.memo.clone())
        .bind(&tags_json)
        .execute(&state.db.0)
        .await;
    if let Err(e) = exec
    {
        tracing::error!(error = ?e, "failed to insert hand");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "db_error" })),
        )
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

pub async fn list_hands(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(q): Query<ListHandsQuery>,
) -> impl IntoResponse {
    let user_id = user_id_from_headers(&headers);
    let limit = q.limit.unwrap_or(50).min(200) as i64;
    let keyword = q.q.unwrap_or_default();

    let mut query = String::from(
        r#"SELECT id, created_at, result_json, memo, tags_json FROM hands WHERE user_id = ?"#,
    );
    if !keyword.is_empty() {
        query.push_str(" AND (memo LIKE ? OR tags_json LIKE ?)");
    }
    query.push_str(" ORDER BY created_at DESC LIMIT ?");
    let mut qx = sqlx::query(&query).bind(&user_id);
    if !keyword.is_empty() {
        let like = format!("%{}%", keyword);
        qx = qx.bind(like.clone()).bind(like);
    }
    let rows = match qx.bind(limit).fetch_all(&state.db.0).await {
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
    let summaries: Vec<HandSummary> = rows.iter().filter_map(row_to_hand_summary).collect();

    (StatusCode::OK, Json(summaries)).into_response()
}

const GET_HAND_SQL: &str = r#"
SELECT id, user_id, created_at, rule_set_id, calc_version, fact_json, result_json, memo, tags_json
FROM hands
WHERE user_id = ? AND id = ? LIMIT 1
"#;

pub async fn get_hand(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user_id = user_id_from_headers(&headers);

    let not_found = (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "not_found" })),
    )
        .into_response();

    let row = match sqlx::query(GET_HAND_SQL)
        .bind(&user_id)
        .bind(&id)
        .fetch_optional(&state.db.0)
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
        return not_found;
    };
    match row_to_hand_record(&row) {
        Some(record) => (StatusCode::OK, Json(record)).into_response(),
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "parse_error" })),
        )
            .into_response(),
    }
}

const RECALC_SQL: &str =
    "SELECT calc_version, fact_json, result_json FROM hands WHERE user_id = ? AND id = ? LIMIT 1";

pub async fn recalc_hand(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user_id = user_id_from_headers(&headers);

    let row = match sqlx::query(RECALC_SQL)
        .bind(&user_id)
        .bind(&id)
        .fetch_optional(&state.db.0)
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
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "not_found" })),
        )
            .into_response();
    };
    let stored_calc_version: String = row.try_get("calc_version").unwrap();
    let fact_json: String = row.try_get("fact_json").unwrap();
    let result_json: String = row.try_get("result_json").unwrap();

    let fact: Fact = match serde_json::from_str(&fact_json) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "corrupt_fact" })),
            )
                .into_response()
        }
    };
    let stored_result: CalcResult = match serde_json::from_str(&result_json) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "corrupt_result" })),
            )
                .into_response()
        }
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
