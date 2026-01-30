//! 麻雀点数計算ドメイン（Pure / Validation）
//!
//! - **Fact**: 観測された事実（入力経路に依存しない）
//! - **Validation**: 整合性チェック
//! - **Pure calc**: 副作用ゼロの点数計算
//! - **analyze**: 手牌文字列から翻・符・役を算出（牌リスト → Fact 候補）

pub mod analyze;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const RULE_SET_ID_V1: &str =
    "riichi-4p-aka-honba300-no-kiriage-no-kazoe-no-doubleyakuman-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WinMethod {
    Tsumo,
    Ron,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WinnerRole {
    Dealer,
    NonDealer,
}

/// ロンの時の放銃者。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Discarder {
    Opponent1,
    Opponent2,
    Opponent3,
}

/// 入力（Fact）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    pub winner_role: WinnerRole,
    pub win_method: WinMethod,
    pub han: u8,
    pub fu: u16,
    pub honba: u8,
    pub riichi_sticks: u8,
    pub discarder: Option<Discarder>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedFact(pub Fact);

#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "detail", rename_all = "snake_case")]
pub enum ValidationError {
    #[error("翻は1以上で入力してください")]
    HanMustBePositive,
    #[error("符が不正です（20/25/30/40/50...110）")]
    FuInvalid,
    #[error("本場が不正です（0以上）")]
    HonbaInvalid,
    #[error("供託が不正です（0以上）")]
    RiichiSticksInvalid,
    #[error("ロンの場合は放銃者が必須です")]
    RonRequiresDiscarder,
    #[error("ツモの場合は放銃者は指定できません")]
    TsumoCannotHaveDiscarder,
}

pub fn validate_fact(fact: Fact) -> Result<ValidatedFact, Vec<ValidationError>> {
    let mut errors = vec![];

    if fact.han == 0 {
        errors.push(ValidationError::HanMustBePositive);
    }

    if !is_valid_fu(fact.fu) {
        errors.push(ValidationError::FuInvalid);
    }

    if fact.honba > 100 {
        errors.push(ValidationError::HonbaInvalid);
    }
    if fact.riichi_sticks > 100 {
        errors.push(ValidationError::RiichiSticksInvalid);
    }

    match (fact.win_method, fact.discarder) {
        (WinMethod::Ron, None) => errors.push(ValidationError::RonRequiresDiscarder),
        (WinMethod::Tsumo, Some(_)) => errors.push(ValidationError::TsumoCannotHaveDiscarder),
        _ => {}
    }

    if errors.is_empty() {
        Ok(ValidatedFact(fact))
    } else {
        Err(errors)
    }
}

fn is_valid_fu(fu: u16) -> bool {
    matches!(fu, 20 | 25)
        || (fu >= 30 && fu <= 110 && fu % 10 == 0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LimitKind {
    Regular,
    Mangan,
    Haneman,
    Baiman,
    Sanbaiman,
    Yakuman,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// 符・翻から算出した基礎点
    pub base_points: u32,
    pub limit: LimitKind,
    /// 上限適用前の基礎点
    pub raw_base_points: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentLine {
    #[serde(rename = "from")]
    pub from_: Participant,
    #[serde(rename = "to")]
    pub to_: Participant,
    pub points: i32,
    pub reason: PaymentReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentReason {
    HandPoints,
    Honba,
    RiichiPot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Participant {
    Winner,
    Opponent1,
    Opponent2,
    Opponent3,
    Pot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalcResult {
    pub rule_set_id: String,
    pub breakdown: ScoreBreakdown,
    pub display_hand_points: String,
    pub payments: Vec<PaymentLine>,
    pub deltas: Vec<(Participant, i32)>,
    pub totals: Totals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Totals {
    pub hand_points_total_to_winner: i32,
    pub honba_points_total_to_winner: i32,
    pub riichi_points_total_to_winner: i32,
    pub total_to_winner: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct RuleSet {
    pub rule_set_id: &'static str,
    pub honba_total_points: u32, // 300
    pub honba_tsumo_each: u32,   // 100
    pub riichi_stick: u32,       // 1000
}

pub const RULESET_V1: RuleSet = RuleSet {
    rule_set_id: RULE_SET_ID_V1,
    honba_total_points: 300,
    honba_tsumo_each: 100,
    riichi_stick: 1000,
};

pub fn calc_score(valid: &ValidatedFact, rules: RuleSet) -> CalcResult {
    let fact = &valid.0;
    let breakdown = calc_breakdown_from_fu_han(fact.fu, fact.han);

    let (display_hand_points, mut hand_payments) = calc_hand_points_payments(
        fact.winner_role,
        fact.win_method,
        breakdown.base_points,
        fact.discarder,
    );

    // 本場
    let mut honba_payments: Vec<PaymentLine> = vec![];
    if fact.honba > 0 {
        let honba = fact.honba as i32;
        match fact.win_method {
            WinMethod::Tsumo => {
                for opp in [Participant::Opponent1, Participant::Opponent2, Participant::Opponent3]
                {
                    honba_payments.push(PaymentLine {
                        from_: opp,
                        to_: Participant::Winner,
                        points: (rules.honba_tsumo_each as i32) * honba,
                        reason: PaymentReason::Honba,
                    });
                }
            }
            WinMethod::Ron => {
                // ロン：放銃者が+300（honba_total_points）を支払う
                let discarder = match fact.discarder {
                    Some(Discarder::Opponent1) => Participant::Opponent1,
                    Some(Discarder::Opponent2) => Participant::Opponent2,
                    Some(Discarder::Opponent3) => Participant::Opponent3,
                    None => Participant::Opponent1, // validate済みのため到達しない
                };
                honba_payments.push(PaymentLine {
                    from_: discarder,
                    to_: Participant::Winner,
                    points: (rules.honba_total_points as i32) * honba,
                    reason: PaymentReason::Honba,
                });
            }
        }
    }

    // 供託（総取り、支払い元はPotとして表現）
    let mut riichi_payments: Vec<PaymentLine> = vec![];
    if fact.riichi_sticks > 0 {
        riichi_payments.push(PaymentLine {
            from_: Participant::Pot,
            to_: Participant::Winner,
            points: (rules.riichi_stick as i32) * (fact.riichi_sticks as i32),
            reason: PaymentReason::RiichiPot,
        });
    }

    let mut payments = vec![];
    payments.append(&mut hand_payments);
    payments.extend(honba_payments);
    payments.extend(riichi_payments);

    let deltas = aggregate_deltas(&payments);

    let hand_points_total_to_winner = payments
        .iter()
        .filter(|p| p.to_ == Participant::Winner && p.reason == PaymentReason::HandPoints)
        .map(|p| p.points)
        .sum();
    let honba_points_total_to_winner = payments
        .iter()
        .filter(|p| p.to_ == Participant::Winner && p.reason == PaymentReason::Honba)
        .map(|p| p.points)
        .sum();
    let riichi_points_total_to_winner = payments
        .iter()
        .filter(|p| p.to_ == Participant::Winner && p.reason == PaymentReason::RiichiPot)
        .map(|p| p.points)
        .sum();
    let total_to_winner = hand_points_total_to_winner
        + honba_points_total_to_winner
        + riichi_points_total_to_winner;

    CalcResult {
        rule_set_id: rules.rule_set_id.to_string(),
        breakdown,
        display_hand_points,
        payments,
        deltas,
        totals: Totals {
            hand_points_total_to_winner,
            honba_points_total_to_winner,
            riichi_points_total_to_winner,
            total_to_winner,
        },
    }
}

fn calc_breakdown_from_fu_han(fu: u16, han: u8) -> ScoreBreakdown {
    // 仕様：数え役満なし。
    let (limit, base_points, raw_base_points) = if han >= 13 {
        (LimitKind::Sanbaiman, 6000, 6000u32)
    } else if han >= 11 {
        (LimitKind::Sanbaiman, 6000, 6000u32)
    } else if han >= 8 {
        (LimitKind::Baiman, 4000, 4000u32)
    } else if han >= 6 {
        (LimitKind::Haneman, 3000, 3000u32)
    } else if han == 5 {
        (LimitKind::Mangan, 2000, 2000u32)
    } else {
        let exp = (han as u32).saturating_add(2);
        let raw = (fu as u32).saturating_mul(2u32.saturating_pow(exp.min(31)));
        if raw >= 2000 {
            (LimitKind::Mangan, 2000, 2000u32)
        } else {
            (LimitKind::Regular, raw, raw)
        }
    };

    ScoreBreakdown {
        base_points,
        limit,
        raw_base_points,
    }
}

fn ceil_to_100(points: u32) -> u32 {
    ((points + 99) / 100) * 100
}

fn calc_hand_points_payments(
    winner_role: WinnerRole,
    win_method: WinMethod,
    base_points: u32,
    discarder: Option<Discarder>,
) -> (String, Vec<PaymentLine>) {
    match (winner_role, win_method) {
        (WinnerRole::Dealer, WinMethod::Ron) => {
            let ron_points = ceil_to_100(base_points * 6);
            let discarder = match discarder {
                Some(Discarder::Opponent1) => Participant::Opponent1,
                Some(Discarder::Opponent2) => Participant::Opponent2,
                Some(Discarder::Opponent3) => Participant::Opponent3,
                None => Participant::Opponent1, // validate済みのため到達しない
            };
            (
                format!("{ron_points}"),
                vec![PaymentLine {
                    from_: discarder,
                    to_: Participant::Winner,
                    points: ron_points as i32,
                    reason: PaymentReason::HandPoints,
                }],
            )
        }
        (WinnerRole::NonDealer, WinMethod::Ron) => {
            let ron_points = ceil_to_100(base_points * 4);
            let discarder = match discarder {
                Some(Discarder::Opponent1) => Participant::Opponent1,
                Some(Discarder::Opponent2) => Participant::Opponent2,
                Some(Discarder::Opponent3) => Participant::Opponent3,
                None => Participant::Opponent1, // validate済みのため到達しない
            };
            (
                format!("{ron_points}"),
                vec![PaymentLine {
                    from_: discarder,
                    to_: Participant::Winner,
                    points: ron_points as i32,
                    reason: PaymentReason::HandPoints,
                }],
            )
        }
        (WinnerRole::Dealer, WinMethod::Tsumo) => {
            let each = ceil_to_100(base_points * 2);
            let payments = [Participant::Opponent1, Participant::Opponent2, Participant::Opponent3]
                .into_iter()
                .map(|opp| PaymentLine {
                    from_: opp,
                    to_: Participant::Winner,
                    points: each as i32,
                    reason: PaymentReason::HandPoints,
                })
                .collect::<Vec<_>>();
            (format!("{each}オール"), payments)
        }
        (WinnerRole::NonDealer, WinMethod::Tsumo) => {
            // MVPでは「Opponent1を親」「Opponent2/3を子」として扱う（UI側で明示）
            let from_dealer = ceil_to_100(base_points * 2);
            let from_non_dealer = ceil_to_100(base_points * 1);
            let payments = vec![
                PaymentLine {
                    from_: Participant::Opponent1, // 親
                    to_: Participant::Winner,
                    points: from_dealer as i32,
                    reason: PaymentReason::HandPoints,
                },
                PaymentLine {
                    from_: Participant::Opponent2,
                    to_: Participant::Winner,
                    points: from_non_dealer as i32,
                    reason: PaymentReason::HandPoints,
                },
                PaymentLine {
                    from_: Participant::Opponent3,
                    to_: Participant::Winner,
                    points: from_non_dealer as i32,
                    reason: PaymentReason::HandPoints,
                },
            ];
            (
                format!("{from_non_dealer}/{from_dealer}"),
                payments,
            )
        }
    }
}

fn aggregate_deltas(payments: &[PaymentLine]) -> Vec<(Participant, i32)> {
    let delta = |p: Participant| -> i32 {
        let incoming: i32 = payments
            .iter()
            .filter(|x| x.to_ == p)
            .map(|x| x.points)
            .sum();
        let outgoing: i32 = payments
            .iter()
            .filter(|x| x.from_ == p)
            .map(|x| x.points)
            .sum();
        incoming - outgoing
    };

    [
        Participant::Winner,
        Participant::Opponent1,
        Participant::Opponent2,
        Participant::Opponent3,
        Participant::Pot,
    ]
    .into_iter()
    .map(|p| (p, delta(p)))
    .collect()
}
