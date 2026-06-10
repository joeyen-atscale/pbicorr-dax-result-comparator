# pbicorr-dax-result-comparator

DAX-semantic result comparator: given two `ResultValue`s (actual vs expected) and a `TolerancePolicy`, returns `Equal`, `WithinTolerance`, or `Mismatch` with a typed `MismatchReason`.

## Purpose

A converted measure is "correct" only if its executed result matches the Power BI reference under DAX semantics — and DAX semantics are not naive float equality. `BLANK()` is not `0`, a decimal measure rounds at a declared scale, dates compare at a boundary, a table result is order-insensitive. This crate is the shared verdict engine used by the round-trip harness and LLM-tail eval.

## Key API

- `compare(actual, expected, policy) -> Verdict`
- `Verdict`: `Equal` | `WithinTolerance { detail }` | `Mismatch { reason }`
- `MismatchReason`: `BlankVsZero`, `BlankVsValue`, `NumericBeyondTolerance`, `ScaleDiffers`, `TypeDiffers`, `DateBoundary`, `RowCountDiffers`, `RowSetDiffers`, `CellMismatch`, `TextDiffers`, `BooleanDiffers`, `RowOrderDiffers`
- `TolerancePolicy`: abs/rel epsilon, ULP budget, `treat_blank_as_zero`, decimal scale, date granularity, order sensitivity

## Acceptance Criteria

1. `compare(BLANK, Number(0), default)` → `Mismatch(BlankVsZero)`; with `treat_blank_as_zero=true` → `Equal`
2. Numbers within epsilon/ULP → `WithinTolerance`; beyond → `Mismatch(NumericBeyondTolerance)` with values
3. Scalar vs Table → `Mismatch(TypeDiffers)`, never panic
4. Tables with identical rows in different order → `Equal` (default); `Mismatch` when `order_sensitive=true`
5. Missing row → `RowCountDiffers`/`RowSetDiffers` naming the key; wrong cell → `CellMismatch` with row + column
6. Same-day timestamps → `Equal` at `granularity=day`; `Mismatch(DateBoundary)` at `granularity=second`
7. Reflexivity: `compare(x, x, default)` is `Equal` for all `ResultValue`s

## Install

Add to `Cargo.toml` (once published to crates.io, or via git path):

```toml
[dependencies]
pbicorr-dax-result-comparator = "0.1"
```

Or, from the Git repo directly:

```toml
[dependencies]
pbicorr-dax-result-comparator = { git = "https://github.com/joeyen-atscale/pbicorr-dax-result-comparator" }
```

Quick example:

```rust
use pbicorr_dax_result_comparator::{compare, ResultValue, TolerancePolicy, Verdict, MismatchReason};

let policy = TolerancePolicy::default();
assert!(matches!(
    compare(&ResultValue::Blank, &ResultValue::Number(0.0), &policy),
    Verdict::Mismatch { reason: MismatchReason::BlankVsZero }
));
```

## License

MIT OR Apache-2.0
