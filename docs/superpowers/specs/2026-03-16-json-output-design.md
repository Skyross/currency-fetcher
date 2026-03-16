# JSON Output Support — Design Spec

**Date:** 2026-03-16
**Status:** Approved

## Overview

Add a `--format` CLI flag that switches output between the existing ASCII table and a flat JSON array. No changes to fetch logic; output format is purely a presentation concern.

## CLI

`OutputFormat` is defined in `main.rs` (alongside `Cli`). It derives `clap::ValueEnum`, `Clone`, and `Copy`. `ValueEnum` is covered by the existing `clap = { features = ["derive"] }` — no new features needed:

```rust
#[derive(clap::ValueEnum, Clone, Copy)]
pub enum OutputFormat {
    Table,
    Json,
}
```

New argument on `Cli`:
```rust
#[arg(short = 'f', long, default_value = "table")]
format: OutputFormat,
```

Because `OutputFormat` is defined in `main.rs` (the crate root), it is already in scope there — no `use` statement needed in `main.rs`. `output.rs` references it as `crate::OutputFormat`.

Example usage:
```
cargo run -- --format json
cargo run -- -c usd,eur -b poland --format json
```

## Models

Current state of `models.rs` (confirmed by reading the file):
- `Country` and `Currency` are unit enums (no fields, no tuple/struct variants) with no existing serde attributes.
- `ExchangeRate` has fields: `country: Country`, `currency: Currency`, `rate: f64`, `date: String`. No existing serde attributes.
- `quick-xml` serde integration is used only in the `banks/` modules (XML deserialization of API responses), not on these model types.

Changes: add `#[derive(serde::Serialize)]` — and only `Serialize`, not `Deserialize` — to all three types in a single edit. No attribute conflicts to resolve.

`Country` and `Currency` also get `#[serde(rename_all = "lowercase")]`. For unit enums this renames the variant tag to lowercase: `Country::Belarus` → `"belarus"`, `Currency::USD` → `"usd"`. This affects JSON output only; `FromStr` handles input parsing independently.

`date` is already a `String` — no format annotation needed.

`rate` is `f64`. `serde_json::to_string_pretty` returns `Err` for `NaN`/`Infinity` (which are not valid JSON numbers); `.unwrap()` will then panic. This is acceptable: `NaN`/`Infinity` from a bank API indicates a broken upstream response, and a panic is the appropriate signal.

## Output Module (`src/output.rs`)

New file. `main.rs` gains `mod output;` alongside `mod banks; mod display; mod models;`.

`output.rs`:
```rust
use crate::display;
use crate::models::ExchangeRate;
use crate::OutputFormat;

pub fn print_rates(rates: &[ExchangeRate], format: OutputFormat) {
    match format {
        OutputFormat::Table => display::print_rates(rates),
        OutputFormat::Json => {
            // Panics on NaN/Infinity — acceptable, indicates broken upstream data
            println!("{}", serde_json::to_string_pretty(rates).unwrap());
        }
    }
}
```

`display::print_rates` has signature `pub fn print_rates(rates: &[ExchangeRate])` and handles an empty slice by printing `"No rates fetched."`. The JSON path produces `[]` for an empty slice — this asymmetry is intentional; `[]` is the correct machine-readable representation.

`display.rs` is otherwise untouched.

## JSON Shape

Flat array of objects, one per rate:

```json
[
  {"country": "belarus", "currency": "usd", "rate": 3.2345, "date": "2026-03-16"},
  {"country": "poland",  "currency": "usd", "rate": 4.0123, "date": "2026-03-16"}
]
```

Empty result: `[]`.

## `main.rs` Changes

1. Define `OutputFormat` enum before `Cli`
2. Add `mod output;`
3. Add `format: OutputFormat` field to `Cli`
4. Replace `display::print_rates(&all_rates)` with `output::print_rates(&all_rates, cli.format)` — `Copy` on `OutputFormat` means no move issue

## Dependencies

No new crates. `serde` (with `derive`), `serde_json`, and `clap` (with `derive`) are already in `Cargo.toml`.

## Out of Scope

- Streaming / line-delimited JSON
- CSV or other formats
- Error output in JSON (fetch failures still go to stderr as plain text warnings)
