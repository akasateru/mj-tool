use axum::http::HeaderMap;
use mj_domain::{CalcResult, Fact};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CalcRequest {
    pub fact: Fact,
}

#[derive(Debug, Serialize)]
pub struct CalcResponse {
    pub calc_version: String,
    pub result: CalcResult,
}

#[derive(Debug, Deserialize)]
pub struct CreateHandRequest {
    pub fact: Fact,
    pub memo: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct HandRecord {
    pub id: Uuid,
    pub user_id: String,
    pub created_at: OffsetDateTime,
    pub rule_set_id: String,
    pub calc_version: String,
    pub fact: Fact,
    pub result: CalcResult,
    pub memo: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListHandsQuery {
    pub q: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct HandSummary {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub display_hand_points: String,
    pub total_to_winner: i32,
    pub memo: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RecalcResponse {
    pub stored_calc_version: String,
    pub current_calc_version: String,
    pub same: bool,
    pub stored_result: CalcResult,
    pub current_result: CalcResult,
}

pub fn user_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "local".to_string())
}
