//! [`ResultValue`] — the discriminated union of values DAX expressions can produce.

use std::collections::BTreeMap;

/// Granularity for date comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateGranularity {
    /// Compare only the calendar date (year-month-day). Times are ignored.
    Day,
    /// Compare down to the second.
    Second,
}

/// An opaque key identifying a row in a table result.
///
/// Constructed from the grain-column values of that row, formatted as
/// `"col1=val1,col2=val2"` in sorted column order so comparisons are
/// deterministic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RowKey(pub String);

/// A single cell value within a table row.
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    /// NULL / BLANK cell.
    Blank,
    /// Numeric cell (int, decimal, float — all unified as f64 for comparison).
    Number(f64),
    /// Text cell.
    Text(String),
    /// Boolean cell.
    Boolean(bool),
    /// Date/datetime cell stored as Unix epoch seconds (UTC).
    Date(i64),
}

/// A table result: rows keyed by their grain-column identity.
///
/// `columns` is the ordered list of non-key column names. Each row in `rows`
/// maps column name → cell value. Rows with the same key are a PRD violation;
/// the comparator handles duplicate keys conservatively (first wins).
#[derive(Debug, Clone)]
pub struct TableResult {
    /// Ordered column names (excluding grain/key columns).
    pub columns: Vec<String>,
    /// Row data. Keys are grain-column identifiers.
    pub rows: BTreeMap<RowKey, BTreeMap<String, CellValue>>,
}

/// The discriminated union of values a DAX expression can return.
///
/// This mirrors the corpus crate's `ResultValue`. Until that crate ships a
/// stable public API this crate re-declares the minimal subset it needs.
#[derive(Debug, Clone)]
pub enum ResultValue {
    /// `BLANK()` — distinct from `0`, `""`, and `false` under DAX semantics.
    Blank,
    /// Numeric scalar (integer, decimal, or float).
    Number(f64),
    /// Text scalar.
    Text(String),
    /// Boolean scalar.
    Boolean(bool),
    /// Date/datetime stored as Unix epoch seconds (UTC).
    Date(i64),
    /// A table result (e.g. from a table expression or `SUMMARIZE`).
    Table(TableResult),
}
