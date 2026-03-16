# JSON Output Support Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `--format` flag to the CLI that emits exchange rates as a flat JSON array instead of the default ASCII table.

**Architecture:** `OutputFormat` enum lives in `main.rs` alongside `Cli`. A new `src/output.rs` module owns format dispatch, delegating to existing `display::print_rates` for table output and calling `serde_json::to_string_pretty` for JSON. `models.rs` gets `serde::Serialize` on all three types.

**Tech Stack:** `serde` + `serde_json` (already in `Cargo.toml`), `clap` `ValueEnum` (already in `Cargo.toml` via `derive` feature).

---

## Chunk 1: Serializable models and JSON helper

### Task 1: Add `serde::Serialize` to model types

**Files:**
- Modify: `src/models.rs`

- [ ] **Step 1: Write the failing test**

Add an inline test module at the bottom of `src/models.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exchange_rate_serializes_to_flat_json_shape() {
        let rate = ExchangeRate {
            country: Country::Poland,
            currency: Currency::USD,
            rate: 4.0123,
            date: "2026-03-16".to_string(),
        };
        let json = serde_json::to_string(&rate).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["country"], "poland");
        assert_eq!(v["currency"], "usd");
        assert_eq!(v["rate"], 4.0123);
        assert_eq!(v["date"], "2026-03-16");
    }
}
```

- [ ] **Step 2: Run the test to confirm it fails**

```bash
cargo test exchange_rate_serializes_to_flat_json_shape
```

Expected: compile error — `ExchangeRate` does not implement `Serialize`.

- [ ] **Step 3: Add `Serialize` derives and serde rename attributes**

In `src/models.rs`, make the following changes:

Add `use serde::Serialize;` at the top, after the existing `use std::fmt;`:
```rust
use serde::Serialize;
```

Add derive and attribute to `Currency`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
```

Add derive and attribute to `Country`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Country {
```

Add derive to `ExchangeRate`:
```rust
#[derive(Debug, Clone, Serialize)]
pub struct ExchangeRate {
```

- [ ] **Step 4: Run the test to confirm it passes**

```bash
cargo test exchange_rate_serializes_to_flat_json_shape
```

Expected: `test models::tests::exchange_rate_serializes_to_flat_json_shape ... ok`

- [ ] **Step 5: Commit**

```bash
git add src/models.rs
git commit -m "feat: add Serialize to model types for JSON output"
```

---

### Task 2: Create `src/output.rs` with `format_json` helper

**Files:**
- Create: `src/output.rs`

Note: Only `format_json` (the private JSON-formatting helper) is added in this task. The public `print_rates` dispatcher, which depends on `OutputFormat` from `main.rs`, is added in Task 3.

- [ ] **Step 1: Write the failing test**

Create `src/output.rs` with only the test module:

```rust
#[cfg(test)]
mod tests {
    use crate::models::{Country, Currency, ExchangeRate};
    use super::*;

    fn make_rates() -> Vec<ExchangeRate> {
        vec![
            ExchangeRate {
                country: Country::Poland,
                currency: Currency::USD,
                rate: 4.0123,
                date: "2026-03-16".to_string(),
            },
            ExchangeRate {
                country: Country::Belarus,
                currency: Currency::EUR,
                rate: 3.1200,
                date: "2026-03-16".to_string(),
            },
        ]
    }

    #[test]
    fn format_json_produces_pretty_json_array() {
        let rates = make_rates();
        let json = format_json(&rates);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["country"], "poland");
        assert_eq!(parsed[0]["currency"], "usd");
        assert_eq!(parsed[1]["country"], "belarus");
        assert_eq!(parsed[1]["currency"], "eur");
    }

    #[test]
    fn format_json_empty_slice_produces_empty_array() {
        let json = format_json(&[]);
        assert_eq!(json.trim(), "[]");
    }
}
```

- [ ] **Step 2: Add `mod output;` to `main.rs`**

In `src/main.rs`, add alongside the existing module declarations:
```rust
mod output;
```

- [ ] **Step 3: Run to confirm test fails**

```bash
cargo test format_json
```

Expected: compile error — `format_json` not defined (the test module compiles but `format_json` is unresolved).

- [ ] **Step 4: Write the `format_json` implementation above the test module**

Prepend to `src/output.rs` so the file is ordered: `use` imports, then functions, then `#[cfg(test)]`:

```rust
use crate::models::ExchangeRate;

pub(crate) fn format_json(rates: &[ExchangeRate]) -> String {
    serde_json::to_string_pretty(rates).unwrap()
}
```

- [ ] **Step 5: Run the tests to confirm they pass**

```bash
cargo test format_json
```

Expected:
```
test output::tests::format_json_empty_slice_produces_empty_array ... ok
test output::tests::format_json_produces_pretty_json_array ... ok
```

- [ ] **Step 6: Run all tests to confirm nothing regressed**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/output.rs src/main.rs
git commit -m "feat: add output module with format_json helper"
```

---

## Chunk 2: CLI wiring

### Task 3: Define `OutputFormat`, add `print_rates` dispatcher, wire up `--format` flag

**Files:**
- Modify: `src/main.rs`
- Modify: `src/output.rs`

- [ ] **Step 1: Define `OutputFormat` in `main.rs`**

In `src/main.rs`, add immediately before the `#[derive(Parser)]` block. Use `pub(crate)` so `output.rs` can reference it as `crate::OutputFormat`:

```rust
#[derive(clap::ValueEnum, Clone, Copy)]
pub(crate) enum OutputFormat {
    Table,
    Json,
}
```

- [ ] **Step 2: Add `format` field to `Cli`**

In the `Cli` struct, add after the `banks` field:

```rust
/// Output format
#[arg(short = 'f', long, default_value = "table")]
format: OutputFormat,
```

- [ ] **Step 3: Add `print_rates` to `output.rs`**

Move all `use` declarations to the top of `src/output.rs` and add `print_rates` after `format_json`, before the `#[cfg(test)]` block. The final file structure is:

```rust
use crate::display;
use crate::models::ExchangeRate;
use crate::OutputFormat;

pub fn print_rates(rates: &[ExchangeRate], format: OutputFormat) {
    match format {
        OutputFormat::Table => display::print_rates(rates),
        OutputFormat::Json => {
            // Panics on NaN/Infinity — acceptable, indicates broken upstream data
            println!("{}", format_json(rates));
        }
    }
}

pub(crate) fn format_json(rates: &[ExchangeRate]) -> String {
    serde_json::to_string_pretty(rates).unwrap()
}

#[cfg(test)]
mod tests {
    // ... existing tests unchanged
}
```

`OutputFormat` is defined in `main.rs` (the crate root), so `crate::OutputFormat` resolves correctly from `output.rs`.

- [ ] **Step 4: Replace the `display::print_rates` call in `main.rs`**

In `src/main.rs`, replace:
```rust
display::print_rates(&all_rates);
```

with:
```rust
output::print_rates(&all_rates, cli.format);
```

- [ ] **Step 5: Build to confirm it compiles**

```bash
cargo build
```

Expected: compiles without errors or warnings.

- [ ] **Step 6: Run all tests**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 7: Smoke test table output (default)**

```bash
cargo run -- -c usd -b poland
```

Expected: ASCII table with a Poland/USD row.

- [ ] **Step 8: Smoke test JSON output**

```bash
cargo run -- -c usd -b poland --format json
```

Expected: pretty-printed JSON array:
```json
[
  {
    "country": "poland",
    "currency": "usd",
    "rate": <number>,
    "date": "<date>"
  }
]
```

- [ ] **Step 9: Smoke test empty result in JSON mode**

```bash
cargo run -- --format json -b poland -c gbp
```

Expected: stdout is `[]` and any warning goes to stderr. (NBP may not publish GBP; the important check is that stdout is valid JSON, not a human-readable message.)

- [ ] **Step 10: Commit**

```bash
git add src/main.rs src/output.rs
git commit -m "feat: add --format flag to CLI for JSON output"
```
