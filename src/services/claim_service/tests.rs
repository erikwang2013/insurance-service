use super::*;

fn d(n: i64) -> Decimal {
    Decimal::from(n)
}

fn date(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).expect("测试日期合法")
}

#[test]
fn accident_date_must_fall_inside_coverage_period() {
    let start = date(2026, 1, 1);
    let end = date(2026, 12, 31);
    let today = date(2026, 6, 1);
    // 缺省用今天；首尾日含在保障期内
    assert!(validate_accident_window(None, Some(start), Some(end), today).is_ok());
    assert!(validate_accident_window(Some(start), Some(start), Some(end), today).is_ok());
    assert!(validate_accident_window(Some(end), Some(start), Some(end), today).is_ok());
    // 保障期外
    assert!(
        validate_accident_window(Some(date(2025, 12, 31)), Some(start), Some(end), today)
            .is_err()
    );
    assert!(
        validate_accident_window(Some(date(2027, 1, 1)), Some(start), Some(end), today).is_err()
    );
    // 缺省日期为今天，但今天不在保障期内
    assert!(
        validate_accident_window(None, Some(start), Some(end), date(2027, 1, 1)).is_err()
    );
    // 保障期缺失或倒置
    assert!(validate_accident_window(Some(start), None, Some(end), today).is_err());
    assert!(validate_accident_window(Some(start), Some(end), Some(start), today).is_err());
}

#[test]
fn claim_amount_must_be_positive_and_within_insurance_amount() {
    assert!(validate_claim_amount(d(1), d(100)).is_ok());
    assert!(validate_claim_amount(d(100), d(100)).is_ok());
    assert!(validate_claim_amount(Decimal::ZERO, d(100)).is_err());
    assert!(validate_claim_amount(d(-1), d(100)).is_err());
    assert!(validate_claim_amount(d(101), d(100)).is_err());
}

#[test]
fn approved_amount_must_be_positive_and_within_caps() {
    assert!(validate_approved_amount(d(50), d(80), d(100)).is_ok());
    assert!(validate_approved_amount(d(80), d(80), d(100)).is_ok());
    assert!(validate_approved_amount(Decimal::ZERO, d(80), d(100)).is_err());
    assert!(validate_approved_amount(d(81), d(80), d(100)).is_err());
    // 申请金额低于核定上限但保额低于核定金额
    assert!(validate_approved_amount(d(95), d(200), d(90)).is_err());
}
