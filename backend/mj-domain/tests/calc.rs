//! 点数計算のテスト（rstest で parametrize 風に 1 ケース = 1 テスト）

use mj_domain::{
    calc_score, validate_fact, Discarder, Fact, LimitKind, RULESET_V1, WinMethod, WinnerRole,
};
use rstest::rstest;

fn fact(
    winner_role: WinnerRole,
    win_method: WinMethod,
    han: u8,
    fu: u16,
    honba: u8,
    riichi_sticks: u8,
    discarder: Option<Discarder>,
) -> Fact {
    Fact {
        winner_role,
        win_method,
        han,
        fu,
        honba,
        riichi_sticks,
        discarder,
    }
}

/// 基本ケース：表示と合計受取（1 ケースごとに別テスト）
#[rstest]
#[case::child_ron_30_4(
    fact(WinnerRole::NonDealer, WinMethod::Ron, 4, 30, 0, 0, Some(Discarder::Opponent1)),
    "7700",
    7700
)]
#[case::dealer_tsumo_40_3(
    fact(WinnerRole::Dealer, WinMethod::Tsumo, 3, 40, 0, 0, None),
    "2600オール",
    2600 * 3
)]
#[case::child_tsumo_30_4(
    fact(WinnerRole::NonDealer, WinMethod::Tsumo, 4, 30, 0, 0, None),
    "2000/3900",
    2000 + 3900 + 2000
)]
#[case::dealer_ron_40_3(
    fact(WinnerRole::Dealer, WinMethod::Ron, 3, 40, 0, 0, Some(Discarder::Opponent1)),
    "7700",
    7700
)]
#[case::honba_1(
    fact(WinnerRole::NonDealer, WinMethod::Ron, 4, 30, 1, 0, Some(Discarder::Opponent1)),
    "7700",
    7700 + 300
)]
#[case::riichi_stick_1(
    fact(WinnerRole::NonDealer, WinMethod::Ron, 4, 30, 0, 1, Some(Discarder::Opponent1)),
    "7700",
    7700 + 1000
)]
fn calc_display_and_total(
    #[case] f: Fact,
    #[case] display: &str,
    #[case] total_to_winner: i32,
) {
    let v = validate_fact(f).unwrap();
    let r = calc_score(&v, RULESET_V1);
    assert_eq!(r.display_hand_points, display);
    assert_eq!(r.totals.total_to_winner, total_to_winner);
}

/// 満貫以上：limit と表示（1 ケースごとに別テスト）
#[rstest]
#[case::mangan(
    fact(WinnerRole::NonDealer, WinMethod::Ron, 4, 40, 0, 0, Some(Discarder::Opponent2)),
    "8000",
    8000,
    LimitKind::Mangan
)]
#[case::haneman(
    fact(WinnerRole::NonDealer, WinMethod::Ron, 6, 30, 0, 0, Some(Discarder::Opponent1)),
    "12000",
    12000,
    LimitKind::Haneman
)]
#[case::baiman(
    fact(WinnerRole::NonDealer, WinMethod::Ron, 8, 30, 0, 0, Some(Discarder::Opponent1)),
    "16000",
    16000,
    LimitKind::Baiman
)]
#[case::sanbaiman(
    fact(WinnerRole::NonDealer, WinMethod::Ron, 13, 30, 0, 0, Some(Discarder::Opponent3)),
    "24000",
    24000,
    LimitKind::Sanbaiman
)]
fn calc_limit_and_display(
    #[case] f: Fact,
    #[case] display: &str,
    #[case] total_to_winner: i32,
    #[case] limit: LimitKind,
) {
    let v = validate_fact(f).unwrap();
    let r = calc_score(&v, RULESET_V1);
    assert_eq!(r.display_hand_points, display);
    assert_eq!(r.totals.total_to_winner, total_to_winner);
    assert_eq!(r.breakdown.limit, limit);
}

/// 20符・25符は有効（1 符ごとに別テスト）
#[rstest]
#[case(20)]
#[case(25)]
fn calc_accepts_fu_20_and_25(#[case] fu: u16) {
    let f = fact(
        WinnerRole::NonDealer,
        WinMethod::Ron,
        1,
        fu,
        0,
        0,
        Some(Discarder::Opponent1),
    );
    let v = validate_fact(f).unwrap();
    let r = calc_score(&v, RULESET_V1);
    assert_eq!(r.breakdown.raw_base_points, (fu as u32) * 2u32.pow(3));
}
