# pbicorr-dax-result-comparator

A verdict engine that decides whether a converted measure's result matches its Power BI reference under DAX semantics — not under float equality. Given two `ResultValue`s and a `TolerancePolicy`, `compare` returns `Equal`, `WithinTolerance`, or `Mismatch` with a typed reason.

## Why it exists

"Correct" is the hard word in a measure-conversion harness. A converted DAX measure is correct only when its executed result matches the Power BI reference, and DAX disagrees with `==` on what "matches" means. `BLANK()` is a distinct value from `0`. A decimal measure agrees at its declared scale and nowhere finer. Two timestamps on the same calendar day are the same day. A table result has no inherent row order unless the query imposed one. Float equality gets every one of these wrong — too strict where DAX is forgiving, too loose where DAX is exact.

So the comparison itself is the thing worth getting right and worth sharing. This crate holds that one decision, with the DAX rules encoded once, so the round-trip harness and the LLM-tail eval both judge correctness the same way.

## What it does

`compare(actual, expected, policy) -> Verdict` is total: every input pair yields a verdict, and the function never panics.

```rust
use pbicorr_dax_result_comparator::{compare, MismatchReason, ResultValue, TolerancePolicy, Verdict};

let policy = TolerancePolicy::default();

// BLANK() is not 0 under DAX — default policy says so.
let v = compare(&ResultValue::Blank, &ResultValue::Number(0.0), &policy);
assert!(matches!(v, Verdict::Mismatch { reason: MismatchReason::BlankVsZero }));
```

The verdict carries its evidence. A `Mismatch` names exactly what diverged:

| Reason | When |
| --- | --- |
| `BlankVsZero` | one side is `BLANK`, the other is `0`, and `treat_blank_as_zero` is off |
| `BlankVsValue` | one side is `BLANK`, the other a non-zero value |
| `NumericBeyondTolerance` | two numbers differ past every tolerance budget — carries `actual`, `expected`, `allowed_abs` |
| `ScaleDiffers` | numbers disagree at the declared decimal scale |
| `TypeDiffers` | mismatched value kinds, e.g. scalar vs table |
| `DateBoundary` | dates differ after truncation to the policy's granularity |
| `RowCountDiffers` / `RowSetDiffers` | tables differ in row count, or in which row keys are present |
| `CellMismatch` | a shared row disagrees in one cell — carries the row key and column |
| `TextDiffers` / `BooleanDiffers` | scalar text or boolean values disagree |
| `RowOrderDiffers` | rows are in a different order under an order-sensitive policy |

A `WithinTolerance` carries a `detail` string describing how much of the tolerance budget the difference consumed.

## The tolerance policy

`TolerancePolicy` is where the DAX rules become knobs. The defaults are deliberately tight; loosen only what a given measure's semantics ask you to.

| Field | Default | Meaning |
| --- | --- | --- |
| `abs_epsilon` | `1e-9` | absolute float tolerance |
| `rel_epsilon` | `1e-9` | relative float tolerance |
| `ulp_budget` | `4` | numbers within this many ULPs count as within tolerance |
| `treat_blank_as_zero` | `false` | when true, `BLANK` compares equal to `0` |
| `decimal_scale` | `None` | round to N decimals before comparing (declared-scale measures) |
| `date_granularity` | `Day` | compare dates at `Day` or `Second` |
| `order_sensitive` | `false` | when true, table rows must appear in the same order |
| `max_cell_mismatches` | `10` | cap on reported cell mismatches per table comparison |

`ResultValue` is the union of what a DAX expression can return: `Blank`, `Number(f64)`, `Text`, `Boolean`, `Date(i64)` (Unix epoch seconds, UTC), and `Table`. A `TableResult` keys its rows by their grain-column identity, so row-set comparison is order-insensitive by default and the verdict can name a row precisely.

## Install

The crate is `publish = false`, so depend on it by git, not from crates.io:

```toml
[dependencies]
pbicorr-dax-result-comparator = { git = "https://github.com/joeyen-atscale/pbicorr-dax-result-comparator" }
```

Build and run the tests from a clone:

```sh
cargo build
cargo test
```

The suite is the specification: six acceptance files, one per behavior the harness depends on (BLANK-vs-zero, numeric tolerance, type differs, table order, row/cell mismatch, date granularity), plus property tests asserting totality and reflexivity — `compare(x, x, default)` is `Equal` for every value.

## Where it fits

Part of the [mqo-mcp](https://github.com/joeyen-atscale/mqo-mcp) fleet — the AtScale MQO/MCP engine for AI analytics. This crate is the shared correctness oracle: the round-trip conversion harness and the LLM-tail eval both call `compare` so a measure is judged correct by the same DAX rules everywhere.

## License

MIT OR Apache-2.0
