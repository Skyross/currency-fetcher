mod banks;
mod display;
mod models;
mod output;

use clap::Parser;
use models::{Country, Currency};
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
    all_rates.sort_by(|a, b| {
        a.country
            .to_string()
            .cmp(&b.country.to_string())
            .then(a.currency.to_string().cmp(&b.currency.to_string()))
    });

    display::print_rates(&all_rates);
    Ok(())
}
