# JSON Output Support — Design Spec

**Date:** 2026-03-16
**Status:** Approved

## Overview

Add a `--format` CLI flag that switches output between the existing ASCII table and a flat JSON array. No changes to fetch logic; output format is purely a presentation concern.

## CLI

New `--format` / `-f` argument on the `Cli` struct using a `clap::ValueEnum`:

```
currency-fetcher [OPTIONS]

Options:
  -f, --format <FORMAT>  Output format [default: table] [possible values: table, json]
```

Example usage:
```
cargo run -- --format json
cargo run -- -c usd,eur -b poland --format json
```

## Models

Add `serde::Serialize` to `ExchangeRate`, `Country`, and `Currency` in `models.rs`.

- `Country` and `Currency` serialize as lowercase strings (`"belarus"`, `"usd"`) to match the existing CLI input convention.
- `rate` serializes as `f64` — consumers control decimal formatting.
- `date` serializes as a plain string (already stored as one).

## Output Module (`src/output.rs`)

New module owns format dispatch. Exposes one public function:

```rust
pub fn print_rates(rates: &[ExchangeRate], format: OutputFormat)
```

Internally:
- `OutputFormat::Table` → delegates to existing `display::print_rates`
- `OutputFormat::Json` → `serde_json::to_string_pretty(rates)` printed to stdout

`display.rs` is untouched.

## JSON Shape

Flat array of objects, one per rate:

```json
[
  {"country": "belarus", "currency": "usd", "rate": 3.2345, "date": "2026-03-16"},
  {"country": "poland",  "currency": "usd", "rate": 4.0123, "date": "2026-03-16"}
]
```

## `main.rs` Changes

- Import `OutputFormat` from `output` module (or define in `main.rs` and pass through)
- Replace `display::print_rates(&all_rates)` with `output::print_rates(&all_rates, cli.format)`

## Dependencies

No new crates needed. `serde` and `serde_json` are already in `Cargo.toml`.

## Out of Scope

- Streaming / line-delimited JSON
- CSV or other formats
- Error output in JSON (failures still go to stderr as warnings)
