//! AC3: Scalar vs Table returns `Mismatch(TypeDiffers)`, never panics.

use pbicorr_dax_result_comparator::{
    compare, MismatchReason, ResultValue, TableResult, TolerancePolicy, Verdict,
};
use std::collections::BTreeMap;

fn empty_table() -> ResultValue {
    ResultValue::Table(TableResult {
        columns: vec![],
        rows: BTreeMap::new(),
    })
}

#[test]
fn scalar_vs_table_returns_type_differs() {
    let policy = TolerancePolicy::default();
    let v = compare(&ResultValue::Number(1.0), &empty_table(), &policy);
    assert!(
        matches!(
            v,
            Verdict::Mismatch {
                reason: MismatchReason::TypeDiffers { .. }
            }
        ),
        "expected TypeDiffers, got {v:?}"
    );
}

#[test]
fn table_vs_scalar_returns_type_differs() {
    let policy = TolerancePolicy::default();
    let v = compare(&empty_table(), &ResultValue::Text("hello".into()), &policy);
    assert!(
        matches!(
            v,
            Verdict::Mismatch {
                reason: MismatchReason::TypeDiffers { .. }
            }
        ),
        "expected TypeDiffers, got {v:?}"
    );
}

#[test]
fn blank_vs_table_returns_mismatch() {
    let policy = TolerancePolicy::default();
    let v = compare(&ResultValue::Blank, &empty_table(), &policy);
    // Any mismatch is acceptable; it must not panic
    assert!(matches!(v, Verdict::Mismatch { .. }));
}

#[test]
fn bool_vs_number_returns_type_differs() {
    let policy = TolerancePolicy::default();
    let v = compare(&ResultValue::Boolean(true), &ResultValue::Number(1.0), &policy);
    assert!(
        matches!(
            v,
            Verdict::Mismatch {
                reason: MismatchReason::TypeDiffers { .. }
            }
        ),
        "expected TypeDiffers, got {v:?}"
    );
}
