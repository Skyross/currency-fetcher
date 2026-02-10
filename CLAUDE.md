# currency-fetcher - Development Notes

## Architecture

- Enum dispatch for bank providers (no trait objects)
- Partial failure model: each bank fetches independently
- Concurrent fetches via `futures::future::join_all`
- Shared `reqwest::Client` with 15s timeout

## Bank API Notes

- **NBRB**: per-currency JSON requests, divide `Cur_OfficialRate` by `Cur_Scale`
- **NBG**: per-currency JSON requests (multi-currency in one URL returns empty), response is `[{date, currencies: [{code, rate, quantity}]}]`
- **NBP**: per-currency JSON, `mid` is already per-unit rate
- **CBR**: single XML request, comma decimals (`Value` uses `,`), `DD.MM.YYYY` dates, windows-1251 encoding (ASCII-safe for our fields)

## Build & Run

```
cargo build
cargo run
cargo run -- -c usd -b poland
```
