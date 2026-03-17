use crate::models::{Country, Currency, ExchangeRate};
use reqwest::Client;
use serde::Deserialize;
use super::util;

#[derive(Deserialize)]
struct NbgResponse {
    date: String,
    currencies: Vec<NbgCurrency>,
}

#[derive(Deserialize)]
struct NbgCurrency {
    code: String,
    quantity: f64,
    rate: f64,
}

pub async fn fetch(client: &Client, currencies: &[Currency]) -> anyhow::Result<Vec<ExchangeRate>> {
    let mut rates = Vec::new();
    for &cur in currencies {
        let url = format!(
            "https://nbg.gov.ge/gw/api/ct/monetarypolicy/currencies/en/json/?currencies={}",
            cur
        );
        let items: Vec<NbgResponse> = client.get(&url).send().await?.json().await?;
        if let Some(resp) = items.first() {
            for c in &resp.currencies {
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
                    date: util::trim_date(&resp.date),
                });
            }
        }
    }
    Ok(rates)
}
