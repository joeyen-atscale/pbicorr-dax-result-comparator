//! AC2: Numbers within epsilon → `WithinTolerance`; beyond → `Mismatch(NumericBeyondTolerance)`.

use pbicorr_dax_result_comparator::{compare, MismatchReason, ResultValue, TolerancePolicy, Verdict};

#[test]
fn numbers_within_epsilon_return_within_tolerance() {
    // Use an explicit epsilon large enough to cover the 1e-7 difference.
    // The PRD AC2 example ("1.0000001 vs 1.0 within epsilon") implies the
    // policy's epsilon is set to cover that gap, not that the default policy
    // does so (the default is intentionally tight at 1e-9).
    let policy = TolerancePolicy {
        abs_epsilon: 1e-6,
        rel_epsilon: 1e-6,
        ..TolerancePolicy::default()
    };
    let v = compare(
        &ResultValue::Number(1.000_000_1),
        &ResultValue::Number(1.0),
        &policy,
    );
    assert!(
        matches!(v, Verdict::WithinTolerance { .. }),
        "expected WithinTolerance, got {v:?}"
    );
}

#[test]
fn numbers_beyond_epsilon_return_mismatch() {
    let policy = TolerancePolicy {
        abs_epsilon: 1e-9,
        rel_epsilon: 1e-9,
        ulp_budget: 0,
        ..TolerancePolicy::default()
    };
    let v = compare(
        &ResultValue::Number(2.0),
        &ResultValue::Number(1.0),
        &policy,
    );
    assert!(
        matches!(
            v,
            Verdict::Mismatch {
                reason: MismatchReason::NumericBeyondTolerance { .. }
            }
        ),
        "expected NumericBeyondTolerance, got {v:?}"
    );
}

#[test]
fn numeric_beyond_tolerance_carries_values() {
    let policy = TolerancePolicy {
        abs_epsilon: 0.001,
        rel_epsilon: 0.001,
        ulp_budget: 0,
        ..TolerancePolicy::default()
    };
    let v = compare(
        &ResultValue::Number(10.0),
        &ResultValue::Number(11.0),
        &policy,
    );
    match v {
        Verdict::Mismatch {
            reason: MismatchReason::NumericBeyondTolerance { actual, expected, allowed_abs },
        } => {
            assert!((actual - 10.0).abs() < 1e-12);
            assert!((expected - 11.0).abs() < 1e-12);
            assert!((allowed_abs - 0.001).abs() < 1e-12);
        }
        other => panic!("expected NumericBeyondTolerance, got {other:?}"),
    }
}

#[test]
fn identical_numbers_are_equal() {
    let policy = TolerancePolicy::default();
    let v = compare(&ResultValue::Number(3.14), &ResultValue::Number(3.14), &policy);
    assert!(matches!(v, Verdict::Equal), "expected Equal, got {v:?}");
}
