//! `pbicorr-dax-result-comparator` — DAX-semantic result comparator.
//!
//! Compares two [`ResultValue`]s (actual vs expected) under a [`TolerancePolicy`]
//! and returns a typed [`Verdict`]: [`Verdict::Equal`], [`Verdict::WithinTolerance`],
//! or [`Verdict::Mismatch`] with a [`MismatchReason`].
//!
//! # Design invariants
//!
//! - **Total**: every input pair yields a `Verdict`; the function never panics.
//! - **No float `==`**: numeric comparisons use epsilon / ULP / absolute-error checks.
//! - **BLANK ≠ 0 by default**: `treat_blank_as_zero` is `false`; DAX semantics
//!   treat `BLANK()` as a distinct value from `0`.
//!
//! # Examples
//!
//! ```rust
//! use pbicorr_dax_result_comparator::{
//!     compare, MismatchReason, ResultValue, TolerancePolicy, Verdict,
//! };
//!
//! // BLANK vs 0 → Mismatch under default policy
//! let policy = TolerancePolicy::default();
//! let v = compare(&ResultValue::Blank, &ResultValue::Number(0.0), &policy);
//! assert!(matches!(v, Verdict::Mismatch { reason: MismatchReason::BlankVsZero }));
//!
//! // BLANK vs 0 → Equal when treat_blank_as_zero = true
//! let lenient = TolerancePolicy { treat_blank_as_zero: true, ..TolerancePolicy::default() };
//! let v2 = compare(&ResultValue::Blank, &ResultValue::Number(0.0), &lenient);
//! assert!(matches!(v2, Verdict::Equal));
//!
//! // Near-equal numbers → WithinTolerance (with a policy whose epsilon covers the gap)
//! let wide = TolerancePolicy { abs_epsilon: 1e-6, rel_epsilon: 1e-6, ..TolerancePolicy::default() };
//! let v3 = compare(&ResultValue::Number(1.000_000_1), &ResultValue::Number(1.0), &wide);
//! assert!(matches!(v3, Verdict::WithinTolerance { .. }));
//! ```

#![cfg_attr(not(test), forbid(unsafe_code))]
#![warn(missing_docs)]

mod compare_impl;
mod policy;
mod reason;
mod result_value;
pub(crate) mod table;
mod verdict;

pub use compare_impl::compare;
pub use policy::TolerancePolicy;
pub use reason::MismatchReason;
pub use result_value::{CellValue, DateGranularity, ResultValue, RowKey, TableResult};
pub use verdict::Verdict;
