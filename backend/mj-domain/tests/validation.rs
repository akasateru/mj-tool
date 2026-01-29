//! 入力（Fact）の検証テスト（rstest で parametrize 風に 1 ケース = 1 テスト）

use mj_domain::{
    validate_fact, Discarder, Fact, ValidationError, WinMethod, WinnerRole,
};
use rstest::rstest;

fn fact_ron(han: u8, fu: u16, discarder: Option<Discarder>) -> Fact {
    Fact {
        winner_role: WinnerRole::NonDealer,
        win_method: WinMethod::Ron,
        han,
        fu,
        honba: 0,
        riichi_sticks: 0,
        discarder,
    }
}

/// 不正な入力はエラーになる（1 ケースごとに別テストとして実行される）
#[rstest]
#[case::han_zero(fact_ron(0, 30, Some(Discarder::Opponent1)), ValidationError::HanMustBePositive)]
#[case::invalid_fu(fact_ron(1, 22, Some(Discarder::Opponent1)), ValidationError::FuInvalid)]
#[case::ron_no_discarder(fact_ron(1, 30, None), ValidationError::RonRequiresDiscarder)]
#[case::tsumo_with_discarder(
    Fact {
        winner_role: WinnerRole::NonDealer,
        win_method: WinMethod::Tsumo,
        han: 1,
        fu: 30,
        honba: 0,
        riichi_sticks: 0,
        discarder: Some(Discarder::Opponent1),
    },
    ValidationError::TsumoCannotHaveDiscarder
)]
fn validation_rejects_invalid_inputs(
    #[case] f: Fact,
    #[case] expected: ValidationError,
) {
    let r = validate_fact(f);
    assert!(r.is_err());
    assert!(r.unwrap_err().iter().any(|e| std::mem::discriminant(e) == std::mem::discriminant(&expected)));
}

/// 有効な符は受理される（1 符ごとに別テスト）
#[rstest]
#[case(20)]
#[case(25)]
#[case(30)]
#[case(40)]
#[case(50)]
#[case(110)]
fn validation_accepts_valid_fu(#[case] fu: u16) {
    let f = fact_ron(1, fu, Some(Discarder::Opponent1));
    assert!(validate_fact(f).is_ok());
}
