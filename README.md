# currency-fetcher

CLI tool that fetches official exchange rates (USD, EUR, GBP) from national banks of Belarus, Georgia, Poland, and Russia.

## Supported Banks

| Country  | Bank | API Format |
|----------|------|------------|
| Belarus  | NBRB | JSON       |
| Georgia  | NBG  | JSON       |
| Poland   | NBP  | JSON       |
| Russia   | CBR  | XML        |

All rates are normalized to "national currency per 1 unit of foreign currency".

## Usage

```
currency-fetcher [OPTIONS]

Options:
  -c, --currencies <CURRENCIES>  Currencies to fetch [default: usd,eur,gbp]
  -b, --banks <COUNTRIES>        Banks to query [default: all]
  -h, --help                     Print help
  -V, --version                  Print version
```

### Examples

Fetch all rates:
```
cargo run
```

Single currency, single bank:
```
cargo run -- -c usd -b poland
```

Multiple currencies, specific banks:
```
cargo run -- -c usd,eur -b georgia,russia
```

Country aliases (ISO 2-letter codes) are also supported:
```
cargo run -- -b pl,ge
```

## Building

```
cargo build --release
```

## Partial Failure

Each bank is fetched independently. If one bank is unreachable, the others still display. Failures appear as warnings on stderr.
