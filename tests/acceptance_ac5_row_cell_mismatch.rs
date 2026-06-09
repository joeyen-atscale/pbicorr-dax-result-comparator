//! AC5: Missing row → RowCountDiffers/RowSetDiffers naming the key; wrong cell → CellMismatch.

use pbicorr_dax_result_comparator::{
    compare, CellValue, MismatchReason, ResultValue, RowKey, TableResult, TolerancePolicy, Verdict,
};
use std::collections::BTreeMap;

fn single_row_table(key: &str, col: &str, val: f64) -> ResultValue {
    let mut rows = BTreeMap::new();
    rows.insert(
        RowKey(key.to_owned()),
        [(col.to_owned(), CellValue::Number(val))].into_iter().collect(),
    );
    ResultValue::Table(TableResult {
        columns: vec![col.to_owned()],
        rows,
    })
}

#[test]
fn missing_row_names_the_key() {
    let policy = TolerancePolicy::default();
    let actual = single_row_table("id=A", "revenue", 100.0);
    let mut expected_rows = BTreeMap::new();
    expected_rows.insert(
        RowKey("id=A".to_owned()),
        [("revenue".to_owned(), CellValue::Number(100.0))].into_iter().collect(),
    );
    expected_rows.insert(
        RowKey("id=B".to_owned()),
        [("revenue".to_owned(), CellValue::Number(200.0))].into_iter().collect(),
    );
    let expected = ResultValue::Table(TableResult {
        columns: vec!["revenue".to_owned()],
        rows: expected_rows,
    });

    match compare(&actual, &expected, &policy) {
        Verdict::Mismatch {
            reason: MismatchReason::RowCountDiffers { actual_count, expected_count },
        } => {
            assert_eq!(actual_count, 1);
            assert_eq!(expected_count, 2);
        }
        Verdict::Mismatch {
            reason: MismatchReason::RowSetDiffers { missing, .. },
        } => {
            assert!(missing.contains(&RowKey("id=B".to_owned())));
        }
        other => panic!("expected RowCountDiffers or RowSetDiffers, got {other:?}"),
    }
}

#[test]
fn wrong_cell_reports_cell_mismatch_not_full_dump() {
    let policy = TolerancePolicy::default();
    let actual = single_row_table("id=A", "revenue", 99.0);
    let expected = single_row_table("id=A", "revenue", 100.0);

    match compare(&actual, &expected, &policy) {
        Verdict::Mismatch {
            reason: MismatchReason::CellMismatch { row_key, column },
        } => {
            assert_eq!(row_key, RowKey("id=A".to_owned()));
            assert_eq!(column, "revenue");
        }
        other => panic!("expected CellMismatch, got {other:?}"),
    }
}
