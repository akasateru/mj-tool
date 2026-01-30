//! 手牌解析（牌リスト → 翻・符・役）
//!
//! riichi-tools-rs を利用し、手牌文字列と局状態から翻・符・役リストを算出する。
//! 画像認識の前段として、手入力の牌リストから Fact の候補（han/fu）を出す用途を想定。

use riichi_tools_rs::riichi::table::Table;
use riichi_tools_rs::riichi::yaku::Yaku;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// 手牌解析のリクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeHandRequest {
    /// 手牌文字列。例: "123m456p789s111z22z"（14枚）
    /// 1-9m/p/s = 萬子/筒子/索子、1-7z = 東南西北白發中。槓: 暗槓 (k1m)、明槓 (k4z1)。チー/ポンも可。
    pub hand_string: String,
    #[serde(default)]
    pub riichi: bool,
    #[serde(default = "default_true")]
    pub tsumo: bool,
    /// 場風（1=東, 2=南, 3=西, 4=北）。未指定時は 1
    #[serde(default)]
    pub prevalent_wind: Option<u8>,
    /// 自風（1=東, 2=南, 3=西, 4=北）。未指定時は 1
    #[serde(default)]
    pub seat_wind: Option<u8>,
}

fn default_true() -> bool {
    true
}

/// 手牌解析のレスポンス（API 用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeHandResponse {
    pub han: u8,
    pub fu: u16,
    /// 役名のリスト（日本語または識別子）
    pub yaku: Vec<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AnalyzeHandError {
    #[error("手牌のパースに失敗しました: {0}")]
    ParseFailed(String),
    #[error("和了形ではありません（14枚の有効な手牌を入力してください）")]
    NotWinningHand,
}

/// 手牌文字列と局状態から翻・符・役を算出する。
pub fn analyze_hand(req: &AnalyzeHandRequest) -> Result<AnalyzeHandResponse, AnalyzeHandError> {
    let mut params: HashMap<String, serde_json::Value> = HashMap::new();
    params.insert(
        "my_hand".to_string(),
        serde_json::Value::String(req.hand_string.trim().to_string()),
    );
    params.insert("my_riichi".to_string(), serde_json::Value::Bool(req.riichi));
    params.insert("my_tsumo".to_string(), serde_json::Value::Bool(req.tsumo));
    params.insert(
        "prevalent_wind".to_string(),
        serde_json::Value::Number(serde_json::Number::from(
            req.prevalent_wind.unwrap_or(1).min(4).max(1),
        )),
    );
    params.insert(
        "my_seat_wind".to_string(),
        serde_json::Value::Number(serde_json::Number::from(
            req.seat_wind.unwrap_or(1).min(4).max(1),
        )),
    );

    let map: serde_json::Map<String, serde_json::Value> = params.into_iter().collect();
    let mut table = Table::from_map(&map).map_err(|e| {
        AnalyzeHandError::ParseFailed(format!("{:?}", e))
    })?;

    let (yaku_list, score) = table.yaku().ok_or(AnalyzeHandError::NotWinningHand)?;

    let yaku_names: Vec<String> = yaku_list
        .iter()
        .map(|y| yaku_to_display_name(y))
        .collect();

    Ok(AnalyzeHandResponse {
        han: score.han,
        fu: score.fu as u16,
        yaku: yaku_names,
    })
}

fn yaku_to_display_name(y: &Yaku) -> String {
    use riichi_tools_rs::riichi::yaku::Yaku::*;
    let s = match y {
        MenzenTsumo => "門前清模和",
        Riichi => "立直",
        Ippatsu => "一発",
        Pinfu => "平和",
        Iipeikou => "一盃口",
        Haitei => "海底摸月",
        Houtei => "河底撈魚",
        Rinshan => "嶺上開花",
        Chankan => "槍槓",
        Tanyao => "断幺九",
        EastRound => "場風（東）",
        EastSeat => "自風（東）",
        SouthRound => "場風（南）",
        SouthSeat => "自風（南）",
        WestRound => "場風（西）",
        WestSeat => "自風（西）",
        NorthSeat => "自風（北）",
        WhiteDragons => "白",
        GreenDragons => "發",
        RedDragons => "中",
        DoubleRiichi => "ダブル立直",
        Chanta => "混全帯幺九",
        SanshokuDoujun => "三色同順",
        Ittsu => "一気通貫",
        Toitoi => "対々和",
        Sanankou => "三暗刻",
        SanshokuDoukou => "三色同刻",
        Sankantsu => "三槓子",
        Chiitoitsu => "七対子",
        Honroutou => "混老頭",
        Shousangen => "小三元",
        Honitsu => "混一色",
        Junchan => "純全帯幺九",
        Ryanpeikou => "二盃口",
        Chinitsu => "清一色",
        Kazoe => "数え役満",
        Kokushi => "国士無双",
        Suuankou => "四暗刻",
        Daisangen => "大三元",
        Shousuushii => "小四喜",
        Daisuushii => "大四喜",
        Tsuuiisou => "字一色",
        Chinroutou => "清老頭",
        Ryuuiisou => "緑一色",
        Chuuren => "九蓮宝燈",
        Suukantsu => "四槓子",
        Tenhou => "天和",
        Chiihou => "地和",
    };
    s.to_string()
}
