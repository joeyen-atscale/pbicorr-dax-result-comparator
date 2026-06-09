//! [`MismatchReason`] — typed mismatch discriminants.

use crate::result_value::RowKey;
use thiserror::Error;

/// The reason a comparison returned [`crate::Verdict::Mismatch`].
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum MismatchReason {
    /// Actual is `BLANK`, expected is `Number(0.0)` (or vice-versa),
    /// and `treat_blank_as_zero` is `false`.
    #[error("BLANK vs 0: one operand is BLANK, the other is zero")]
    BlankVsZero,

    /// Actual is `BLANK`, expected is a non-zero value (or vice-versa).
    #[error("BLANK vs value: one operand is BLANK, the other is a non-blank value")]
    BlankVsValue,

    /// Two numeric values differ by more than all configured budgets.
    #[error("numeric beyond tolerance: actual={actual}, expected={expected}, allowed_abs={allowed_abs}")]
    NumericBeyondTolerance {
        /// The actual numeric value.
        actual: f64,
        /// The expected numeric value.
        expected: f64,
        /// The absolute tolerance that was applied.
        allowed_abs: f64,
    },

    /// Two decimal values differ at the declared scale.
    #[error("scale differs: rounded actual={actual_rounded} != rounded expected={expected_rounded} at scale={scale}")]
    ScaleDiffers {
        /// Rounded actual value.
        actual_rounded: f64,
        /// Rounded expected value.
        expected_rounded: f64,
        /// The scale at which rounding was applied.
        scale: u32,
    },

    /// The two values have different types (e.g. Scalar vs Table).
    #[error("type differs: actual_type={actual_type}, expected_type={expected_type}")]
    TypeDiffers {
        /// Description of the actual value's type.
        actual_type: String,
        /// Description of the expected value's type.
        expected_type: String,
    },

    /// Two dates differ after granularity truncation.
    #[error("date boundary: dates differ at the configured granularity")]
    DateBoundary,

    /// Tables have a different number of rows.
    #[error("row count differs: actual={actual_count}, expected={expected_count}")]
    RowCountDiffers {
        /// Row count in the actual result.
        actual_count: usize,
        /// Row count in the expected result.
        expected_count: usize,
    },

    /// Tables share the same row count but differ in which rows are present.
    #[error("row set differs: {missing_count} missing row(s), {unexpected_count} unexpected row(s)")]
    RowSetDiffers {
        /// Keys present in expected but absent in actual.
        missing: Vec<RowKey>,
        /// Keys present in actual but absent in expected.
        unexpected: Vec<RowKey>,
        /// Number of missing rows.
        missing_count: usize,
        /// Number of unexpected rows.
        unexpected_count: usize,
    },

    /// A specific cell in a shared row has a mismatched value.
    #[error("cell mismatch at row={row_key:?}, column={column}")]
    CellMismatch {
        /// The row key identifying the differing row.
        row_key: RowKey,
        /// The column name within that row.
        column: String,
    },

    /// Two text values differ.
    #[error("text differs: actual={actual:?}, expected={expected:?}")]
    TextDiffers {
        /// The actual text.
        actual: String,
        /// The expected text.
        expected: String,
    },

    /// Two boolean values differ.
    #[error("boolean differs: actual={actual}, expected={expected}")]
    BooleanDiffers {
        /// The actual boolean.
        actual: bool,
        /// The expected boolean.
        expected: bool,
    },

    /// Table rows appear in a different order under order-sensitive policy.
    #[error("row order differs under order-sensitive policy")]
    RowOrderDiffers,
}
