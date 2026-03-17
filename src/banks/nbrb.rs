use crate::models::{Country, Currency, ExchangeRate};
use reqwest::Client;
use serde::Deserialize;
use super::util;

#[derive(Deserialize)]
struct NbrbResponse {
    #[serde(rename = "Cur_OfficialRate")]
    official_rate: f64,
    #[serde(rename = "Cur_Scale")]
    scale: f64,
    #[serde(rename = "Date")]
    date: String,
}

fn currency_code(c: Currency) -> &'static str {
    match c {
        Currency::USD => "USD",
        Currency::EUR => "EUR",
        Currency::GBP => "GBP",
    }
}

pub async fn fetch(client: &Client, currencies: &[Currency]) -> anyhow::Result<Vec<ExchangeRate>> {
    let mut rates = Vec::new();
    for &cur in currencies {
        let url = format!(
            "https://api.nbrb.by/exrates/rates/{}?parammode=2",
            currency_code(cur)
        );
        let resp: NbrbResponse = client.get(&url).send().await?.json().await?;
        let date = util::trim_date(&resp.date);
        rates.push(ExchangeRate {
            country: Country::Belarus,
            currency: cur,
            rate: resp.official_rate / resp.scale,
            date,
        });
    }
    Ok(rates)
}
