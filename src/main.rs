mod banks;
mod display; // declared here so output.rs can use crate::display
mod models;
mod output;

use clap::Parser;
use models::{Country, Currency};
use output::OutputFormat;
use reqwest::Client;

#[derive(Parser)]
#[command(name = "currency-fetcher", version, about = "Fetch official exchange rates from national banks")]
struct Cli {
    /// Currencies to fetch (comma-separated: usd,eur,gbp)
    #[arg(short, long, default_value = "usd,eur,gbp")]
    currencies: String,

    /// Banks/countries to query (comma-separated: belarus,georgia,poland,russia or all)
    #[arg(short, long, default_value = "all")]
    banks: String,

    /// Output format
    #[arg(short = 'f', long, default_value = "table")]
    format: OutputFormat,
}

fn parse_currencies(s: &str) -> anyhow::Result<Vec<Currency>> {
    s.split(',')
        .map(|c| c.trim().parse())
        .collect()
}

fn parse_countries(s: &str) -> anyhow::Result<Vec<Country>> {
    if s.trim().eq_ignore_ascii_case("all") {
        return Ok(Country::all().to_vec());
    }
    s.split(',')
        .map(|c| c.trim().parse())
        .collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let currencies = parse_currencies(&cli.currencies)?;
    let countries = parse_countries(&cli.banks)?;

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let futures: Vec<_> = countries
        .iter()
        .map(|&country| {
            let client = client.clone();
            let currencies = currencies.clone();
            async move {
                let result = banks::fetch_rates(&client, country, &currencies).await;
                (country, result)
            }
        })
        .collect();

    let results = futures::future::join_all(futures).await;

    let mut all_rates = Vec::new();
    for (country, result) in results {
        match result {
            Ok(rates) => all_rates.extend(rates),
            Err(e) => eprintln!("Warning: failed to fetch from {country}: {e}"),
        }
    }

    // Sort by country then currency for consistent display
    all_rates.sort_by_key(|r| (r.country, r.currency));

    output::print_rates(&all_rates, cli.format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_currencies_single() {
        let result = parse_currencies("usd").unwrap();
        assert_eq!(result, vec![Currency::USD]);
    }

    #[test]
    fn parse_currencies_multiple() {
        let result = parse_currencies("usd,eur,gbp").unwrap();
        assert_eq!(result, vec![Currency::USD, Currency::EUR, Currency::GBP]);
    }

    #[test]
    fn parse_currencies_with_spaces() {
        let result = parse_currencies(" usd , eur ").unwrap();
        assert_eq!(result, vec![Currency::USD, Currency::EUR]);
    }

    #[test]
    fn parse_currencies_invalid() {
        assert!(parse_currencies("xyz").is_err());
    }

    #[test]
    fn parse_currencies_mixed_valid_invalid() {
        assert!(parse_currencies("usd,xyz").is_err());
    }

    #[test]
    fn parse_countries_all() {
        let result = parse_countries("all").unwrap();
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn parse_countries_all_case_insensitive() {
        let result = parse_countries("ALL").unwrap();
        assert_eq!(result.len(), 4);
        let result2 = parse_countries(" All ").unwrap();
        assert_eq!(result2.len(), 4);
    }

    #[test]
    fn parse_countries_single() {
        let result = parse_countries("poland").unwrap();
        assert_eq!(result, vec![Country::Poland]);
    }

    #[test]
    fn parse_countries_abbreviations() {
        let result = parse_countries("by,ge").unwrap();
        assert_eq!(result, vec![Country::Belarus, Country::Georgia]);
    }

    #[test]
    fn parse_countries_with_spaces() {
        let result = parse_countries(" russia , poland ").unwrap();
        assert_eq!(result, vec![Country::Russia, Country::Poland]);
    }

    #[test]
    fn parse_countries_invalid() {
        assert!(parse_countries("mars").is_err());
    }
}
