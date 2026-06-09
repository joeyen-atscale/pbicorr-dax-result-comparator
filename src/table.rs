//! Table comparison logic.

use std::collections::BTreeMap;

use crate::{
    policy::TolerancePolicy,
    reason::MismatchReason,
    result_value::{CellValue, RowKey, TableResult},
    verdict::Verdict,
};

/// Compare two [`CellValue`]s for equality under the given policy.
///
/// Returns `None` when equal/within-tolerance, `Some(reason)` on mismatch.
fn compare_cell(
    actual: &CellValue,
    expected: &CellValue,
    policy: &TolerancePolicy,
    row_key: &RowKey,
    column: &str,
) -> Option<MismatchReason> {
    let cell_mismatch = || MismatchReason::CellMismatch {
        row_key: row_key.clone(),
        column: column.to_owned(),
    };

    match (actual, expected) {
        (CellValue::Blank, CellValue::Blank) => None,
        (CellValue::Blank, CellValue::Number(n)) | (CellValue::Number(n), CellValue::Blank)
            if policy.treat_blank_as_zero =>
        {
            // Check if the numeric value rounds to zero
            let rounded = policy.round(*n);
            // Intentional exact zero comparison: treat_blank_as_zero means "blank = 0.0 exactly"
            #[allow(clippy::float_cmp)]
            if rounded == 0.0 {
                None
            } else {
                Some(cell_mismatch())
            }
        }
        #[allow(clippy::float_arithmetic)]
        (CellValue::Number(a), CellValue::Number(b)) => {
            let a_r = policy.round(*a);
            let b_r = policy.round(*b);
            if numeric_within_tolerance(a_r, b_r, policy) {
                None
            } else {
                Some(cell_mismatch())
            }
        }
        (CellValue::Text(a), CellValue::Text(b)) => {
            if a == b { None } else { Some(cell_mismatch()) }
        }
        (CellValue::Boolean(a), CellValue::Boolean(b)) => {
            if a == b { None } else { Some(cell_mismatch()) }
        }
        (CellValue::Date(a), CellValue::Date(b)) => {
            if policy.truncate_date(*a) == policy.truncate_date(*b) {
                None
            } else {
                Some(cell_mismatch())
            }
        }
        _ => Some(cell_mismatch()),
    }
}

/// Test whether two (already-rounded) floats are within all configured budgets.
#[allow(clippy::float_arithmetic)] // tolerance arithmetic is the whole point
#[allow(unreachable_pub)] // pub(crate) mod requires pub items to satisfy redundant_pub_crate
pub fn numeric_within_tolerance(a: f64, b: f64, policy: &TolerancePolicy) -> bool {
    let diff = (a - b).abs();
    if diff <= policy.abs_epsilon {
        return true;
    }
    let denom = a.abs().max(b.abs()).max(1.0);
    if diff / denom <= policy.rel_epsilon {
        return true;
    }
    ulp_distance(a, b) <= u64::from(policy.ulp_budget)
}

/// ULP distance between two f64 values.
///
/// Returns `u64::MAX` when signs differ (different order-of-magnitude), since
/// ULP distance is not meaningful across the sign boundary.
///
/// Uses `from_ne_bytes` to reinterpret the bit pattern as `i64` without `as`-cast,
/// avoiding the `cast_possible_wrap` and `as_conversions` lints.
const fn ulp_distance(a: f64, b: f64) -> u64 {
    let ai = i64::from_ne_bytes(a.to_bits().to_ne_bytes());
    let bi = i64::from_ne_bytes(b.to_bits().to_ne_bytes());
    // If signs agree, ULP distance is |ai - bi|
    if (ai < 0) == (bi < 0) {
        ai.abs_diff(bi)
    } else {
        u64::MAX
    }
}

/// Compare two [`TableResult`]s under `policy`.
#[allow(unreachable_pub)] // pub(crate) mod requires pub items to satisfy redundant_pub_crate
pub fn compare_tables(
    actual: &TableResult,
    expected: &TableResult,
    policy: &TolerancePolicy,
) -> Verdict {
    let actual_count = actual.rows.len();
    let expected_count = expected.rows.len();

    if policy.order_sensitive {
        return compare_tables_ordered(actual, expected, policy);
    }

    // Collect keys
    let actual_keys: std::collections::BTreeSet<_> = actual.rows.keys().collect();
    let expected_keys: std::collections::BTreeSet<_> = expected.rows.keys().collect();

    let missing: Vec<RowKey> = expected_keys
        .difference(&actual_keys)
        .map(|k| (*k).clone())
        .collect();
    let unexpected: Vec<RowKey> = actual_keys
        .difference(&expected_keys)
        .map(|k| (*k).clone())
        .collect();

    if !missing.is_empty() || !unexpected.is_empty() {
        if actual_count != expected_count {
            return Verdict::Mismatch {
                reason: MismatchReason::RowCountDiffers {
                    actual_count,
                    expected_count,
                },
            };
        }
        let missing_count = missing.len();
        let unexpected_count = unexpected.len();
        return Verdict::Mismatch {
            reason: MismatchReason::RowSetDiffers {
                missing,
                unexpected,
                missing_count,
                unexpected_count,
            },
        };
    }

    // Shared keys: compare cells
    compare_shared_rows(&actual.rows, &expected.rows, policy)
}

/// Order-sensitive table comparison.
fn compare_tables_ordered(
    actual: &TableResult,
    expected: &TableResult,
    policy: &TolerancePolicy,
) -> Verdict {
    let actual_count = actual.rows.len();
    let expected_count = expected.rows.len();

    if actual_count != expected_count {
        return Verdict::Mismatch {
            reason: MismatchReason::RowCountDiffers {
                actual_count,
                expected_count,
            },
        };
    }

    // Check key order
    let actual_order: Vec<_> = actual.rows.keys().collect();
    let expected_order: Vec<_> = expected.rows.keys().collect();

    if actual_order != expected_order {
        return Verdict::Mismatch {
            reason: MismatchReason::RowOrderDiffers,
        };
    }

    compare_shared_rows(&actual.rows, &expected.rows, policy)
}

/// Compare cell-by-cell for rows that share the same key set.
fn compare_shared_rows(
    actual_rows: &BTreeMap<RowKey, BTreeMap<String, CellValue>>,
    expected_rows: &BTreeMap<RowKey, BTreeMap<String, CellValue>>,
    policy: &TolerancePolicy,
) -> Verdict {
    let empty = BTreeMap::new();
    let mut mismatch_count = 0usize;
    let mut first_mismatch: Option<MismatchReason> = None;

    'outer: for (key, expected_row) in expected_rows {
        let actual_row = actual_rows.get(key).unwrap_or(&empty);
        for (col, expected_cell) in expected_row {
            let actual_cell = actual_row.get(col).unwrap_or(&CellValue::Blank);
            if let Some(reason) = compare_cell(actual_cell, expected_cell, policy, key, col) {
                if first_mismatch.is_none() {
                    first_mismatch = Some(reason);
                }
                mismatch_count += 1;
                if mismatch_count >= policy.max_cell_mismatches {
                    break 'outer;
                }
            }
        }
    }

    first_mismatch.map_or(Verdict::Equal, |reason| Verdict::Mismatch { reason })
}
