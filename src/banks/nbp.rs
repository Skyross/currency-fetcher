use crate::models::{Country, Currency, ExchangeRate};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct NbpResponse {
    rates: Vec<NbpRate>,
}

#[derive(Deserialize)]
struct NbpRate {
    #[serde(rename = "effectiveDate")]
    effective_date: String,
    mid: f64,
}

pub async fn fetch(client: &Client, currencies: &[Currency]) -> anyhow::Result<Vec<ExchangeRate>> {
    let mut rates = Vec::new();
    for &cur in currencies {
        let url = format!(
            "https://api.nbp.pl/api/exchangerates/rates/a/{}/?format=json",
            cur.as_lower_code()
        );
        let resp: NbpResponse = client.get(&url).send().await?.json().await?;
        if let Some(r) = resp.rates.first() {
            rates.push(ExchangeRate {
                country: Country::Poland,
                currency: cur,
                rate: r.mid,
                date: r.effective_date.clone(),
            });
        }
    }
    Ok(rates)
}
