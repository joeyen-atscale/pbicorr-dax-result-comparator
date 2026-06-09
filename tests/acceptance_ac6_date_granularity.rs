//! AC6: `granularity=day` treats same-day timestamps as Equal; `granularity=second` as Mismatch.

use pbicorr_dax_result_comparator::{
    compare, DateGranularity, MismatchReason, ResultValue, TolerancePolicy, Verdict,
};

/// 2026-06-02T09:00:00 UTC as Unix epoch seconds
const TS_0900: i64 = 1_748_858_400;
/// 2026-06-02T17:00:00 UTC as Unix epoch seconds
const TS_1700: i64 = 1_748_887_200;

#[test]
fn same_day_different_time_is_equal_at_day_granularity() {
    let policy = TolerancePolicy {
        date_granularity: DateGranularity::Day,
        ..TolerancePolicy::default()
    };
    let v = compare(
        &ResultValue::Date(TS_0900),
        &ResultValue::Date(TS_1700),
        &policy,
    );
    assert!(
        matches!(v, Verdict::Equal),
        "expected Equal at day granularity, got {v:?}"
    );
}

#[test]
fn same_day_different_time_is_mismatch_at_second_granularity() {
    let policy = TolerancePolicy {
        date_granularity: DateGranularity::Second,
        ..TolerancePolicy::default()
    };
    let v = compare(
        &ResultValue::Date(TS_0900),
        &ResultValue::Date(TS_1700),
        &policy,
    );
    assert!(
        matches!(v, Verdict::Mismatch { reason: MismatchReason::DateBoundary }),
        "expected DateBoundary at second granularity, got {v:?}"
    );
}

#[test]
fn identical_timestamps_are_always_equal() {
    for gran in [DateGranularity::Day, DateGranularity::Second] {
        let policy = TolerancePolicy {
            date_granularity: gran,
            ..TolerancePolicy::default()
        };
        let v = compare(
            &ResultValue::Date(TS_0900),
            &ResultValue::Date(TS_0900),
            &policy,
        );
        assert!(
            matches!(v, Verdict::Equal),
            "expected Equal for identical timestamps at {gran:?}, got {v:?}"
        );
    }
}
