//! [`Verdict`] — the outcome of a [`crate::compare`] call.

use crate::reason::MismatchReason;

/// The result of comparing two [`crate::ResultValue`]s under a [`crate::TolerancePolicy`].
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// The two values are identical under DAX semantics and the given policy.
    Equal,

    /// The two values differ, but the difference is within the policy's tolerance
    /// budgets (epsilon / ULP / decimal scale).
    WithinTolerance {
        /// Human-readable description of how much tolerance was consumed.
        detail: String,
    },

    /// The two values differ beyond the configured tolerance.
    Mismatch {
        /// Typed reason for the mismatch.
        reason: MismatchReason,
    },
}
