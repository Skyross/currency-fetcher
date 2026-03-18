# Coverage & CI Design

## Goal

Max out test coverage using wiremock for async HTTP testing, add GitHub Actions CI with Rust best practices, and add Codecov badge to README.

## 1. Testable Fetch Functions

### Pattern

Each bank module gets an internal `fetch_from(client, base_url, currencies)` that contains all logic. The public `fetch` delegates with the hardcoded production URL. Tests call `fetch_from` with a wiremock mock server URL.

### Per-module changes

**cbr.rs** — Single-request bank (one URL for all currencies):
- `fetch_from(client: &Client, url: &str, currencies: &[Currency])` — fetches XML from exact `url`, delegates to existing `process_xml`
- `fetch` calls `fetch_from` with `"https://www.cbr.ru/scripts/XML_daily.asp"`
- URL pattern: `{url}` (no per-currency suffix)
- Test: wiremock serves XML fixture at `/scripts/XML_daily.asp`, assert correct `ExchangeRate` values

**nbg.rs** — Per-currency requests:
- `fetch_from(client: &Client, base_url: &str, currencies: &[Currency])` — builds per-currency URL as `format!("{}?currencies={}", base_url, cur)`, fetches JSON, delegates to existing `process_response`
- `fetch` calls `fetch_from` with `"https://nbg.gov.ge/gw/api/ct/monetarypolicy/currencies/en/json/"`
- URL pattern: `{base_url}?currencies={cur}` (currency is a **query param**, not path segment)
- Test: wiremock matches with `query_param("currencies", "USD")` etc.

**nbp.rs** — Per-currency requests:
- `fetch_from(client: &Client, base_url: &str, currencies: &[Currency])` — builds URL as `format!("{}/{}/?format=json", base_url, cur.as_lower_code())` (lowercase currency code + `?format=json` query param)
- `fetch` calls `fetch_from` with `"https://api.nbp.pl/api/exchangerates/rates/a"`
- URL pattern: `{base_url}/{lowercase_currency}/?format=json`
- Test: wiremock matches path `/api/exchangerates/rates/a/usd/`

**nbrb.rs** — Per-currency requests:
- `fetch_from(client: &Client, base_url: &str, currencies: &[Currency])` — builds URL as `format!("{}/{}?parammode=2", base_url, cur)` (includes mandatory `?parammode=2` query param)
- `fetch` calls `fetch_from` with `"https://api.nbrb.by/exrates/rates"`
- URL pattern: `{base_url}/{cur}?parammode=2`
- Test: wiremock matches path `/exrates/rates/USD`

### banks/mod.rs

The `fetch_rates` function is a 6-line match dispatch. Each arm is covered when the corresponding bank's `fetch` function is called. Since all four bank `fetch_from` tests exercise their respective arms, `fetch_rates` will be covered indirectly — but only if all four bank test modules exist.

### Dev dependency

```toml
[dev-dependencies]
wiremock = "0.6"
```

### Expected coverage

~85%. Uncovered lines will be:
- `main()` body (async entry point plumbing — convention not to unit-test)
- The thin `fetch` wrappers that call `fetch_from` with hardcoded production URLs (2-3 lines each, 4 modules = ~12 lines)

## 2. GitHub Actions CI

### File: `.github/workflows/ci.yml`

Single workflow, 4 jobs:

| Job | Command | Purpose |
|-----|---------|---------|
| `check` | `cargo check` | Compilation validation |
| `test` | `cargo test` | Run test suite |
| `lint` | `cargo fmt --check` + `cargo clippy -- -D warnings` | Code quality |
| `coverage` | `cargo tarpaulin --out xml` → upload to Codecov | Coverage tracking |

All jobs run on `ubuntu-latest` with stable Rust. Uses `dtolnay/rust-toolchain` action. Triggers on push and pull_request.

### Coverage upload

Uses `codecov/codecov-action@v4` to upload tarpaulin's cobertura XML. Requires `CODECOV_TOKEN` secret (set in GitHub repo settings after signing up at codecov.io).

## 3. README Badge

Add CI status badge and Codecov badge below the `# currency-fetcher` heading:

```markdown
[![CI](https://github.com/Skyross/currency-fetcher/actions/workflows/ci.yml/badge.svg)](https://github.com/Skyross/currency-fetcher/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Skyross/currency-fetcher/branch/master/graph/badge.svg)](https://codecov.io/gh/Skyross/currency-fetcher)
```

## Non-goals

- MSRV policy (no `rust-version` in Cargo.toml currently)
- Cross-platform CI matrix (linux-only CLI)
- Integration tests hitting real bank APIs
- Testing `main()` entry point directly
