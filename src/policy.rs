//! [`TolerancePolicy`] — configurable comparison parameters.

use crate::result_value::DateGranularity;

/// Configures how [`crate::compare`] judges two [`crate::ResultValue`]s.
#[derive(Debug, Clone)]
pub struct TolerancePolicy {
    /// Absolute epsilon for float comparisons: `|a - b| <= abs_epsilon` → within tolerance.
    ///
    /// Default: `1e-9`.
    pub abs_epsilon: f64,

    /// Relative epsilon: `|a - b| / max(|a|, |b|, 1.0) <= rel_epsilon` → within tolerance.
    ///
    /// Default: `1e-9`.
    pub rel_epsilon: f64,

    /// ULP budget: numbers within this many ULPs are considered within tolerance.
    ///
    /// Default: `4`.
    pub ulp_budget: u32,

    /// When `true`, `BLANK` compares equal to `Number(0.0)`.
    ///
    /// Default: `false` — DAX distinguishes BLANK from 0.
    pub treat_blank_as_zero: bool,

    /// Decimal scale: when `Some(n)`, numbers are rounded to `n` decimal places
    /// before comparison (models declared-scale measures).
    ///
    /// Default: `None` (no rounding).
    pub decimal_scale: Option<u32>,

    /// Date comparison granularity.
    ///
    /// Default: [`DateGranularity::Day`].
    pub date_granularity: DateGranularity,

    /// When `true`, table rows must appear in the same order.
    ///
    /// Default: `false` — DAX table results are unordered unless explicitly sorted.
    pub order_sensitive: bool,

    /// Maximum number of cell mismatches to report before stopping.
    ///
    /// Default: `10`.
    pub max_cell_mismatches: usize,
}

impl Default for TolerancePolicy {
    fn default() -> Self {
        Self {
            abs_epsilon: 1e-9,
            rel_epsilon: 1e-9,
            ulp_budget: 4,
            treat_blank_as_zero: false,
            decimal_scale: None,
            date_granularity: DateGranularity::Day,
            order_sensitive: false,
            max_cell_mismatches: 10,
        }
    }
}

/// Seconds per day — used for day-granularity date truncation.
const SECONDS_PER_DAY: i64 = 86_400;

impl TolerancePolicy {
    /// Truncate a Unix-epoch-seconds timestamp to the requested date granularity.
    pub(crate) const fn truncate_date(&self, ts: i64) -> i64 {
        match self.date_granularity {
            DateGranularity::Day => {
                // floor-divide, handling negative timestamps correctly
                ts.div_euclid(SECONDS_PER_DAY) * SECONDS_PER_DAY
            }
            DateGranularity::Second => ts,
        }
    }

    /// Apply `decimal_scale` rounding to a float value.
    #[allow(clippy::float_arithmetic)] // rounding computation is intentional float arithmetic
    pub(crate) fn round(&self, v: f64) -> f64 {
        self.decimal_scale.map_or(v, |scale| {
            let factor = 10_f64.powi(i32::try_from(scale).unwrap_or(i32::MAX));
            (v * factor).round() / factor
        })
    }
}
