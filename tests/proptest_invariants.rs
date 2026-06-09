//! Property-based invariant tests.
//!
//! Read-only after scaffold. The edit-agent must NOT modify proptests.

use pbicorr_dax_result_comparator::{compare, ResultValue, TolerancePolicy, Verdict};
use proptest::prelude::*;

/// Strategy generating scalar ResultValues.
fn arb_scalar() -> impl Strategy<Value = ResultValue> {
    prop_oneof![
        Just(ResultValue::Blank),
        any::<f64>()
            .prop_filter("finite only", |f| f.is_finite())
            .prop_map(ResultValue::Number),
        ".*".prop_map(ResultValue::Text),
        any::<bool>().prop_map(ResultValue::Boolean),
        any::<i64>().prop_map(ResultValue::Date),
    ]
}

proptest! {
    /// AC7: reflexivity — `compare(x, x, default)` is Equal for all scalars.
    ///
    /// Note: NaN is excluded because NaN != NaN; we filter to finite f64 only.
    #[test]
    fn reflexivity(v in arb_scalar()) {
        let policy = TolerancePolicy::default();
        let verdict = compare(&v, &v, &policy);
        prop_assert!(
            matches!(verdict, Verdict::Equal),
            "reflexivity violated for {v:?}: got {verdict:?}"
        );
    }

    /// Blank always mismatches a non-blank non-zero scalar under default policy.
    #[test]
    fn blank_vs_nonblank_nonzero_is_mismatch(n in 1.0_f64..1_000_000.0_f64) {
        let policy = TolerancePolicy::default();
        let verdict = compare(&ResultValue::Blank, &ResultValue::Number(n), &policy);
        prop_assert!(
            matches!(verdict, Verdict::Mismatch { .. }),
            "expected Mismatch, got {verdict:?}"
        );
    }

    /// No compare call panics on any finite-float pair.
    #[test]
    fn no_panic_on_finite_number_pair(a in any::<f64>().prop_filter("finite", |f| f.is_finite()),
                                      b in any::<f64>().prop_filter("finite", |f| f.is_finite())) {
        let policy = TolerancePolicy::default();
        let _verdict = compare(&ResultValue::Number(a), &ResultValue::Number(b), &policy);
    }
}
