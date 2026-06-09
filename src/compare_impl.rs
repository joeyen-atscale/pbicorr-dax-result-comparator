//! Top-level [`compare`] function.

use crate::{
    policy::TolerancePolicy,
    reason::MismatchReason,
    result_value::ResultValue,
    table::{compare_tables, numeric_within_tolerance},
    verdict::Verdict,
};

/// Return the type-name string used in [`MismatchReason::TypeDiffers`].
const fn type_name(v: &ResultValue) -> &'static str {
    match v {
        ResultValue::Blank => "Blank",
        ResultValue::Number(_) => "Number",
        ResultValue::Text(_) => "Text",
        ResultValue::Boolean(_) => "Boolean",
        ResultValue::Date(_) => "Date",
        ResultValue::Table(_) => "Table",
    }
}

/// Compare `actual` against `expected` under `policy`.
///
/// This function is **total**: it returns a [`Verdict`] for every input pair
/// and never panics.
///
/// # Examples
///
/// ```rust
/// use pbicorr_dax_result_comparator::{compare, MismatchReason, ResultValue, TolerancePolicy, Verdict};
///
/// let policy = TolerancePolicy::default();
/// let v = compare(&ResultValue::Blank, &ResultValue::Number(0.0), &policy);
/// assert!(matches!(v, Verdict::Mismatch { reason: MismatchReason::BlankVsZero }));
/// ```
#[must_use]
pub fn compare(actual: &ResultValue, expected: &ResultValue, policy: &TolerancePolicy) -> Verdict {
    match (actual, expected) {
        // ── Blank ──────────────────────────────────────────────────────────
        (ResultValue::Blank, ResultValue::Blank) => Verdict::Equal,

        (ResultValue::Blank, ResultValue::Number(n))
        | (ResultValue::Number(n), ResultValue::Blank) => {
            if policy.treat_blank_as_zero {
                // round before comparing so decimal_scale is respected
                let rounded = policy.round(*n);
                // Safety: comparing rounded float to literal 0.0 is intentional
                // (we want exact zero, not "within epsilon of zero"). This is
                // the one place in the codebase where == on f64 is correct.
                #[allow(clippy::float_cmp)]
                if rounded == 0.0 {
                    Verdict::Equal
                } else {
                    Verdict::Mismatch {
                        reason: MismatchReason::BlankVsValue,
                    }
                }
            } else if *n == 0.0 {
                Verdict::Mismatch {
                    reason: MismatchReason::BlankVsZero,
                }
            } else {
                Verdict::Mismatch {
                    reason: MismatchReason::BlankVsValue,
                }
            }
        }

        (ResultValue::Blank, _) | (_, ResultValue::Blank) => Verdict::Mismatch {
            reason: MismatchReason::BlankVsValue,
        },

        // ── Number ─────────────────────────────────────────────────────────
        (ResultValue::Number(a), ResultValue::Number(b)) => compare_numbers(*a, *b, policy),

        // ── Text ───────────────────────────────────────────────────────────
        (ResultValue::Text(a), ResultValue::Text(b)) => {
            if a == b {
                Verdict::Equal
            } else {
                Verdict::Mismatch {
                    reason: MismatchReason::TextDiffers {
                        actual: a.clone(),
                        expected: b.clone(),
                    },
                }
            }
        }

        // ── Boolean ────────────────────────────────────────────────────────
        (ResultValue::Boolean(a), ResultValue::Boolean(b)) => {
            if a == b {
                Verdict::Equal
            } else {
                Verdict::Mismatch {
                    reason: MismatchReason::BooleanDiffers {
                        actual: *a,
                        expected: *b,
                    },
                }
            }
        }

        // ── Date ───────────────────────────────────────────────────────────
        (ResultValue::Date(a), ResultValue::Date(b)) => {
            if policy.truncate_date(*a) == policy.truncate_date(*b) {
                Verdict::Equal
            } else {
                Verdict::Mismatch {
                    reason: MismatchReason::DateBoundary,
                }
            }
        }

        // ── Table ──────────────────────────────────────────────────────────
        (ResultValue::Table(a), ResultValue::Table(b)) => compare_tables(a, b, policy),

        // ── Type mismatch ──────────────────────────────────────────────────
        _ => Verdict::Mismatch {
            reason: MismatchReason::TypeDiffers {
                actual_type: type_name(actual).to_owned(),
                expected_type: type_name(expected).to_owned(),
            },
        },
    }
}

/// Compare two numeric scalars.
///
/// Extracted from [`compare`] to keep line-count within the clippy threshold.
#[allow(clippy::float_arithmetic)] // floating-point arithmetic is the whole point here
fn compare_numbers(a: f64, b: f64, policy: &TolerancePolicy) -> Verdict {
    let a_r = policy.round(a);
    let b_r = policy.round(b);

    // If a decimal scale is set, check scale-level equality first.
    if let Some(scale) = policy.decimal_scale {
        #[allow(clippy::float_cmp)] // rounded values compared intentionally
        if a_r != b_r {
            // Further check: are they within tolerance anyway?
            if numeric_within_tolerance(a_r, b_r, policy) {
                let diff = (a_r - b_r).abs();
                return Verdict::WithinTolerance {
                    detail: format!(
                        "numeric diff {diff:.2e} within tolerance after scale={scale} rounding"
                    ),
                };
            }
            return Verdict::Mismatch {
                reason: MismatchReason::ScaleDiffers {
                    actual_rounded: a_r,
                    expected_rounded: b_r,
                    scale,
                },
            };
        }
        return Verdict::Equal;
    }

    #[allow(clippy::float_cmp)] // exact equality checked before tolerance path
    if a_r == b_r {
        return Verdict::Equal;
    }

    if numeric_within_tolerance(a_r, b_r, policy) {
        let diff = (a_r - b_r).abs();
        Verdict::WithinTolerance {
            detail: format!("numeric diff {diff:.2e} within tolerance"),
        }
    } else {
        Verdict::Mismatch {
            reason: MismatchReason::NumericBeyondTolerance {
                actual: a,
                expected: b,
                allowed_abs: policy.abs_epsilon,
            },
        }
    }
}
