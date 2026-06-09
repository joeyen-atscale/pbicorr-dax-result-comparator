//! AC1: `compare(BLANK, Number(0), default_policy)` returns `Mismatch(BlankVsZero)`;
//! with `treat_blank_as_zero=true` it returns `Equal`.

use pbicorr_dax_result_comparator::{compare, MismatchReason, ResultValue, TolerancePolicy, Verdict};

#[test]
fn blank_vs_zero_default_policy_is_mismatch() {
    let policy = TolerancePolicy::default();
    let v = compare(&ResultValue::Blank, &ResultValue::Number(0.0), &policy);
    assert!(
        matches!(v, Verdict::Mismatch { reason: MismatchReason::BlankVsZero }),
        "expected BlankVsZero mismatch, got {v:?}"
    );
}

#[test]
fn zero_vs_blank_default_policy_is_mismatch() {
    let policy = TolerancePolicy::default();
    let v = compare(&ResultValue::Number(0.0), &ResultValue::Blank, &policy);
    assert!(
        matches!(v, Verdict::Mismatch { reason: MismatchReason::BlankVsZero }),
        "expected BlankVsZero mismatch, got {v:?}"
    );
}

#[test]
fn blank_vs_zero_treat_as_zero_is_equal() {
    let policy = TolerancePolicy {
        treat_blank_as_zero: true,
        ..TolerancePolicy::default()
    };
    let v = compare(&ResultValue::Blank, &ResultValue::Number(0.0), &policy);
    assert!(
        matches!(v, Verdict::Equal),
        "expected Equal when treat_blank_as_zero=true, got {v:?}"
    );
}

#[test]
fn blank_vs_nonzero_is_blank_vs_value_regardless_of_flag() {
    for flag in [false, true] {
        let policy = TolerancePolicy {
            treat_blank_as_zero: flag,
            ..TolerancePolicy::default()
        };
        let v = compare(&ResultValue::Blank, &ResultValue::Number(42.0), &policy);
        assert!(
            matches!(v, Verdict::Mismatch { reason: MismatchReason::BlankVsValue }),
            "expected BlankVsValue with treat_blank_as_zero={flag}, got {v:?}"
        );
    }
}
