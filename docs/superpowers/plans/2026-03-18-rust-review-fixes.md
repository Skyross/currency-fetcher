# Rust Review Fixes Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers-extended-cc:subagent-driven-development (if subagents available) or
> superpowers-extended-cc:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Apply all actionable findings from the Rust code review — one focused commit per task.

**Architecture:** Each task is a self-contained improvement to an existing file. No new files are created except
`src/banks/util.rs` for shared date parsing. Tasks are ordered so each commit leaves the code in a clean, passing state.

**Tech Stack:** Rust 1.75+, tokio, anyhow, serde, clap, reqwest, quick-xml, tabled

---

## Chunk 1: Correctness & Safety

### Task 1: Fix CBR thousands-separator parsing

**Files:**

- Modify: `src/banks/cbr.rs:24-26`

The current `parse_cbr_decimal` only replaces commas. CBR can emit a non-breaking space (U+00A0) or regular space as a
thousands separator (e.g. `"1 234,56"`), causing the parse to fail and the entire Russia fetch to error.

- [ ] **Step 1: Add a regression test for thousands-separator input**

In `src/banks/cbr.rs`, add a `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cbr_decimal_handles_thousands_separator() {
        assert_eq!(parse_cbr_decimal("1\u{00A0}234,56").unwrap(), 1234.56);
        assert_eq!(parse_cbr_decimal("1 234,56").unwrap(), 1234.56);
        assert_eq!(parse_cbr_decimal("87,6325").unwrap(), 87.6325);
    }
}
```

- [ ] **Step 2: Run the test to confirm it fails**

```bash
cargo test parse_cbr_decimal_handles_thousands_separator 2>&1
```

Expected: FAIL — `invalid float literal` or similar.

- [ ] **Step 3: Fix `parse_cbr_decimal`**

Replace `src/banks/cbr.rs:24-26`:

```rust
fn parse_cbr_decimal(s: &str) -> anyhow::Result<f64> {
    Ok(s.replace('\u{00A0}', "")
        .replace(' ', "")
        .replace(',', ".")
        .parse()?)
}
```

- [ ] **Step 4: Run test to confirm it passes**

```bash
cargo test parse_cbr_decimal_handles_thousands_separator 2>&1
```

Expected: PASS. Then run full suite:

```bash
cargo test 2>&1
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/banks/cbr.rs
git commit -m "fix: strip thousands-separator spaces in CBR decimal parser"
```

---

### Task 2: Parse CBR Nominal as u32

**Files:**

- Modify: `src/banks/cbr.rs:61`

`Nominal` is always a small integer (1, 10, 100). Parsing it as `f64` via `parse_cbr_decimal` is a type mismatch — use
`u32` to avoid floating-point imprecision and make the intent clear.

- [ ] **Step 1: Add test asserting nominal is handled correctly as integer**

In the existing `#[cfg(test)]` block in `src/banks/cbr.rs`, add:

```rust
#[test]
fn parse_nominal_exact() {
    // nominal "10" should divide value "876,3250" to exactly 87.6325
    let nominal: u32 = "10".trim().parse().unwrap();
    let value = parse_cbr_decimal("876,3250").unwrap();
    let result = value / f64::from(nominal);
    assert!((result - 87.6325).abs() < 1e-10);
}
```

- [ ] **Step 2: Run test to confirm it passes already (documents intent)**

```bash
cargo test parse_nominal_exact 2>&1
```

Expected: PASS.

- [ ] **Step 3: Change `nominal` parsing from `parse_cbr_decimal` to `u32`**

In `src/banks/cbr.rs`, inside the `fetch` function, replace:

```rust
        let nominal = parse_cbr_decimal( & v.nominal) ?;
let value = parse_cbr_decimal( & v.value) ?;
rates.push(ExchangeRate {
country: Country::Russia,
currency,
rate: value / nominal,
```

with:

```rust
        let nominal: u32 = v.nominal.trim().parse()
.with_context( | | format!("CBR: invalid nominal '{}'", v.nominal)) ?;
let value = parse_cbr_decimal( & v.value) ?;
rates.push(ExchangeRate {
country: Country::Russia,
currency,
rate: value / f64::from(nominal),
```

Add `use anyhow::Context;` at the top of the file if not present.

- [ ] **Step 4: Run full test suite**

```bash
cargo test 2>&1
```

Expected: all tests pass. Also confirm it compiles cleanly:

```bash
cargo clippy 2>&1
```

- [ ] **Step 5: Commit**

```bash
git add src/banks/cbr.rs
git commit -m "fix: parse CBR Nominal as u32 instead of f64"
```

---

## Chunk 2: Module Structure

### Task 3: Move OutputFormat to output.rs

**Files:**

- Modify: `src/output.rs`
- Modify: `src/main.rs`

`output.rs` currently imports `OutputFormat` from `crate::` (i.e., `main.rs`). The dependency direction is backwards —
`main.rs` should import from `output`, not vice-versa.

- [ ] **Step 1: Add `OutputFormat` definition to `output.rs` and remove `crate::OutputFormat` import**

At the top of `src/output.rs`, replace:

```rust
use crate::display;
use crate::models::ExchangeRate;
use crate::OutputFormat;
```

with:

```rust
use crate::display;
use crate::models::ExchangeRate;

#[derive(clap::ValueEnum, Clone, Copy)]
pub(crate) enum OutputFormat {
    Table,
    Json,
}
```

- [ ] **Step 2: Update `main.rs` to import `OutputFormat` from `output`**

In `src/main.rs`, remove the `OutputFormat` enum definition:

```rust
#[derive(clap::ValueEnum, Clone, Copy)]
pub(crate) enum OutputFormat {
    Table,
    Json,
}
```

And add an import at the top (with the other `use` statements):

```rust
use output::OutputFormat;
```

- [ ] **Step 3: Build to confirm no compilation errors**

```bash
cargo build 2>&1
```

Expected: clean build.

- [ ] **Step 4: Run full test suite**

```bash
cargo test 2>&1
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/output.rs src/main.rs
git commit -m "refactor: move OutputFormat to output.rs where it belongs"
```

---

### Task 4: Restrict display::print_rates visibility to pub(crate)

**Files:**

- Modify: `src/display.rs:16`

`print_rates` in `display.rs` is `pub` but it's only called from within this crate. `pub(crate)` is the correct
visibility.

- [ ] **Step 1: Change `pub fn print_rates` to `pub(crate) fn print_rates` in `display.rs`**

In `src/display.rs`, line 16, change:

```rust
pub fn print_rates(rates: &[ExchangeRate]) {
```

to:

```rust
pub(crate) fn print_rates(rates: &[ExchangeRate]) {
```

- [ ] **Step 2: Build and test**

```bash
cargo build 2>&1 && cargo test 2>&1
```

Expected: clean build, all tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/display.rs
git commit -m "chore: narrow display::print_rates visibility to pub(crate)"
```

---

## Chunk 3: Idiomatic Patterns

### Task 5: Derive Ord on Country/Currency and fix sort

**Files:**

- Modify: `src/models.rs:4-9`, `src/models.rs:36-43`
- Modify: `src/main.rs:80-85`

The sort in `main.rs` calls `.to_string()` twice per comparison step, allocating heap strings. Deriving `PartialOrd`/
`Ord` on the enums gives zero-allocation discriminant-based comparison.

- [ ] **Step 1: Write a test that asserts sort order**

In `src/models.rs`, in the existing `#[cfg(test)]` block, add:

```rust
#[test]
fn country_and_currency_sort_by_declaration_order() {
    assert!(Country::Belarus < Country::Georgia);
    assert!(Country::Georgia < Country::Poland);
    assert!(Country::Poland < Country::Russia);
    assert!(Currency::USD < Currency::EUR);
    assert!(Currency::EUR < Currency::GBP);
}
```

- [ ] **Step 2: Run the test to confirm it fails (Ord not yet derived)**

```bash
cargo test country_and_currency_sort_by_declaration_order 2>&1
```

Expected: FAIL — `binary operation '<' cannot be applied`.

- [ ] **Step 3: Add `PartialOrd, Ord` derives to both enums in `models.rs`**

Change line 4:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
```

to (for `Currency`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
```

And line 36 (for `Country`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
```

- [ ] **Step 4: Simplify the sort in `main.rs`**

Replace lines 80-85:

```rust
    all_rates.sort_by( | a, b| {
a.country
.to_string()
.cmp(& b.country.to_string())
.then(a.currency.to_string().cmp( & b.currency.to_string()))
});
```

with:

```rust
    all_rates.sort_by_key( | r| (r.country, r.currency));
```

- [ ] **Step 5: Run tests**

```bash
cargo test 2>&1
```

Expected: all tests pass including the new sort-order test.

- [ ] **Step 6: Commit**

```bash
git add src/models.rs src/main.rs
git commit -m "refactor: derive Ord on enums, simplify sort to sort_by_key"
```

---

### Task 6: Extract shared parse_date utility

**Files:**

- Create: `src/banks/util.rs`
- Modify: `src/banks/mod.rs`
- Modify: `src/banks/nbg.rs:18-20`
- Modify: `src/banks/nbrb.rs:32`

The same `split('T').next()` date-trimming logic appears in both `nbg.rs` and `nbrb.rs`. Extract it to avoid
duplication.

- [ ] **Step 1: Create `src/banks/util.rs` with the shared helper**

```rust
/// Trims an ISO 8601 datetime string to just the date part.
/// "2024-01-15T00:00:00" → "2024-01-15"
/// Already-date strings are returned unchanged.
pub(super) fn trim_date(s: &str) -> String {
    s.split('T').next().unwrap_or(s).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_date_strips_time_component() {
        assert_eq!(trim_date("2024-01-15T00:00:00"), "2024-01-15");
    }

    #[test]
    fn trim_date_returns_plain_date_unchanged() {
        assert_eq!(trim_date("2024-01-15"), "2024-01-15");
    }
}
```

- [ ] **Step 2: Run the tests to confirm they pass**

```bash
cargo test trim_date 2>&1
```

Expected: FAIL — module not declared yet. That's expected.

- [ ] **Step 3: Declare `util` module in `src/banks/mod.rs`**

In `src/banks/mod.rs`, add `mod util;` at the top:

```rust
mod cbr;
mod nbg;
mod nbp;
mod nbrb;
mod util;
```

- [ ] **Step 4: Run tests again**

```bash
cargo test trim_date 2>&1
```

Expected: PASS.

- [ ] **Step 5: Replace inline parse_date in `nbg.rs`**

Remove the `parse_date` function (lines 18-20) and its call site. Replace:

```rust
fn parse_date(date_str: &str) -> String {
    date_str.split('T').next().unwrap_or(date_str).to_string()
}
```

*(delete this function entirely)*

And in the `fetch` function, replace `parse_date(&resp.date)` with `util::trim_date(&resp.date)`.

Add `use super::util;` at the top of `nbg.rs` after the existing imports.

- [ ] **Step 6: Replace inline date parsing in `nbrb.rs`**

Remove the inline expression on line 32. Replace:

```rust
        let date = resp.date.split('T').next().unwrap_or( & resp.date).to_string();
```

with:

```rust
        let date = util::trim_date( & resp.date);
```

Add `use super::util;` at the top of `nbrb.rs`.

- [ ] **Step 7: Build and test**

```bash
cargo build 2>&1 && cargo test 2>&1
```

Expected: clean build, all tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/banks/util.rs src/banks/mod.rs src/banks/nbg.rs src/banks/nbrb.rs
git commit -m "refactor: extract shared trim_date utility to banks/util.rs"
```

---

### Task 7: Remove redundant currency_code helpers

**Files:**

- Modify: `src/banks/nbrb.rs:15-21`, `src/banks/nbrb.rs:27-29`
- Modify: `src/banks/nbp.rs:17-23`, `src/banks/nbp.rs:29`

Both `nbrb.rs` and `nbp.rs` define a local `currency_code()` that maps `Currency` to a string. The `Currency` `Display`
impl already returns `"USD"`, `"EUR"`, `"GBP"`. Use `.to_string()` directly for NBRB (uppercase), and
`.to_string().to_lowercase()` for NBP.

- [ ] **Step 1: Remove `currency_code` from `nbrb.rs` and inline it in the URL**

Delete lines 15-21 in `nbrb.rs`:

```rust
fn currency_code(c: Currency) -> &'static str {
    match c {
        Currency::USD => "USD",
        Currency::EUR => "EUR",
        Currency::GBP => "GBP",
    }
}
```

In the `fetch` function, replace the URL format:

```rust
            "https://api.nbrb.by/exrates/rates/{}?parammode=2",
currency_code(cur)
```

with:

```rust
            "https://api.nbrb.by/exrates/rates/{}?parammode=2",
cur
```

(`cur` implements `Display` which produces `"USD"` etc.)

- [ ] **Step 2: Remove `currency_code` from `nbp.rs` and inline it**

Delete lines 17-23 in `nbp.rs`:

```rust
fn currency_code(c: Currency) -> &'static str {
    match c {
        Currency::USD => "usd",
        Currency::EUR => "eur",
        Currency::GBP => "gbp",
    }
}
```

In the `fetch` function, replace:

```rust
            "https://api.nbp.pl/api/exchangerates/rates/a/{}/?format=json",
currency_code(cur)
```

with:

```rust
            "https://api.nbp.pl/api/exchangerates/rates/a/{}/?format=json",
cur.to_string().to_lowercase()
```

- [ ] **Step 3: Build and test**

```bash
cargo build 2>&1 && cargo test 2>&1
```

Expected: clean build, all tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/banks/nbrb.rs src/banks/nbp.rs
git commit -m "refactor: remove redundant currency_code helpers, use Display impl"
```

---

### Task 8: Fix NBG redundant code re-match

**Files:**

- Modify: `src/banks/nbg.rs:30-44`

The NBG URL is constructed per-`cur` (one currency per request). The inner loop re-matches `c.code` to determine which
currency it is — but we already know it's `cur`. The `_ => continue` arm silently swallows unexpected codes rather than
surfacing them.

- [ ] **Step 1: Simplify the NBG inner loop to use `cur` directly**

Replace the inner `for c in &resp.currencies` block:

```rust
        if let Some(resp) = items.first() {
for c in & resp.currencies {
let currency = match c.code.as_str() {
"USD" => Currency::USD,
"EUR" => Currency::EUR,
"GBP" => Currency::GBP,
_ => continue,
};
rates.push(ExchangeRate {
country: Country::Georgia,
currency,
rate: c.rate / c.quantity,
date: util::trim_date( & resp.date),
});
}
}
```

with:

```rust
        if let Some(resp) = items.first() {
if let Some(c) = resp.currencies.first() {
rates.push(ExchangeRate {
country: Country::Georgia,
currency: cur,
rate: c.rate / c.quantity,
date: util::trim_date( & resp.date),
});
}
}
```

- [ ] **Step 2: Remove `NbgCurrency.code` field since it's no longer used**

In the `NbgCurrency` struct, remove the `code` field:

```rust
#[derive(Deserialize)]
struct NbgCurrency {
    quantity: f64,
    rate: f64,
}
```

(serde will ignore the `code` field from the JSON response automatically.)

- [ ] **Step 3: Build and test**

```bash
cargo build 2>&1 && cargo test 2>&1
```

Expected: clean build, all tests pass. Check for unused import warnings too:

```bash
cargo clippy 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/banks/nbg.rs
git commit -m "refactor: simplify NBG fetch to use cur directly, remove redundant code re-match"
```

---

## Chunk 4: Minor Polish

### Task 9: Use eq_ignore_ascii_case in Currency::from_str

**Files:**

- Modify: `src/models.rs:27`

`s.to_uppercase().as_str()` allocates a `String` for a 3-character input on every CLI parse. `eq_ignore_ascii_case`
avoids the allocation.

- [ ] **Step 1: Rewrite `Currency::from_str` to use `eq_ignore_ascii_case`**

Replace the `from_str` implementation (lines 26-33):

```rust
    fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_uppercase().as_str() {
        "USD" => Ok(Currency::USD),
        "EUR" => Ok(Currency::EUR),
        "GBP" => Ok(Currency::GBP),
        other => anyhow::bail!("unknown currency: {other}"),
    }
}
```

with:

```rust
    fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.eq_ignore_ascii_case("usd") {
        Ok(Currency::USD)
    } else if s.eq_ignore_ascii_case("eur") {
        Ok(Currency::EUR)
    } else if s.eq_ignore_ascii_case("gbp") {
        Ok(Currency::GBP)
    } else {
        anyhow::bail!("unknown currency: {s}")
    }
}
```

- [ ] **Step 2: Build and test**

```bash
cargo build 2>&1 && cargo test 2>&1
```

Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/models.rs
git commit -m "refactor: use eq_ignore_ascii_case in Currency::from_str to avoid allocation"
```

---

### Task 10: Use HashSet for CBR wanted-currencies lookup

**Files:**

- Modify: `src/banks/cbr.rs:44-60`

`wanted.contains()` on a `Vec<&str>` is O(n) linear scan. For the 3-element case it's trivially fast, but a `HashSet`
makes set-membership intent explicit and is the idiomatic choice.

- [ ] **Step 1: Replace `Vec<&str>` with `HashSet<&str>` in CBR fetch**

In `src/banks/cbr.rs`, add `use std::collections::HashSet;` at the top.

Replace the `wanted` construction and filter:

```rust
    let wanted: Vec< & str> = currencies.iter().map( | c| match c {
Currency::USD => "USD",
Currency::EUR => "EUR",
Currency::GBP => "GBP",
}).collect();
```

with:

```rust
    let wanted: HashSet< & str> = currencies.iter().map( | c| match c {
Currency::USD => "USD",
Currency::EUR => "EUR",
Currency::GBP => "GBP",
}).collect();
```

The `wanted.contains(...)` call on line 52 continues to work unchanged since `HashSet` also implements `contains`.

- [ ] **Step 2: Build and test**

```bash
cargo build 2>&1 && cargo test 2>&1
```

Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/banks/cbr.rs
git commit -m "refactor: use HashSet for CBR wanted-currencies lookup"
```

---

## Summary

| Task | File(s)                              | Type     | Commit message                                              |
|------|--------------------------------------|----------|-------------------------------------------------------------|
| 1    | `cbr.rs`                             | bug fix  | fix: strip thousands-separator spaces in CBR decimal parser |
| 2    | `cbr.rs`                             | fix      | fix: parse CBR Nominal as u32 instead of f64                |
| 3    | `output.rs`, `main.rs`               | refactor | refactor: move OutputFormat to output.rs where it belongs   |
| 4    | `display.rs`                         | chore    | chore: narrow display::print_rates visibility to pub(crate) |
| 5    | `models.rs`, `main.rs`               | refactor | refactor: derive Ord on enums, simplify sort to sort_by_key |
| 6    | `banks/util.rs`, `nbg.rs`, `nbrb.rs` | refactor | refactor: extract shared trim_date utility to banks/util.rs |
| 7    | `nbrb.rs`, `nbp.rs`                  | refactor | refactor: remove redundant currency_code helpers            |
| 8    | `nbg.rs`                             | refactor | refactor: simplify NBG fetch to use cur directly            |
| 9    | `models.rs`                          | refactor | refactor: use eq_ignore_ascii_case in Currency::from_str    |
| 10   | `cbr.rs`                             | refactor | refactor: use HashSet for CBR wanted-currencies lookup      |
