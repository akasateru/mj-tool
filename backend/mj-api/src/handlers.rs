use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mj_domain::{calc_score, validate_fact, CalcResult, Fact, RULESET_V1};
use sqlx::Row;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::{
    user_id_from_headers, CalcRequest, CalcResponse, CreateHandRequest, HandRecord, HandSummary,
    ListHandsQuery, RecalcResponse,
};
use crate::state::AppState;

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "ok": true }))
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

pub async fn get_hand(
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
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "not_found" })),
        )
            .into_response();
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

pub async fn recalc_hand(
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
