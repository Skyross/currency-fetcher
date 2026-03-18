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
    quantity: f64,
    rate: f64,
}

fn process_response(items: &[NbgResponse], cur: Currency) -> Option<ExchangeRate> {
    let resp = items.first()?;
    let c = resp.currencies.first()?;
    Some(ExchangeRate {
        country: Country::Georgia,
        currency: cur,
        rate: c.rate / c.quantity,
        date: util::trim_date(&resp.date),
    })
}

pub(super) async fn fetch(client: &Client, base_url: &str, currencies: &[Currency]) -> anyhow::Result<Vec<ExchangeRate>> {
    let mut rates = Vec::new();
    for &cur in currencies {
        let url = format!("{}?currencies={}", base_url, cur);
        let items: Vec<NbgResponse> = client.get(&url).send().await?.json().await?;
        if let Some(rate) = process_response(&items, cur) {
            rates.push(rate);
        }
    }
    Ok(rates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nbg_response_deserializes() {
        let json = r#"[{"date":"2026-03-18T00:00:00","currencies":[{"code":"USD","quantity":1.0,"rate":2.85}]}]"#;
        let items: Vec<NbgResponse> = serde_json::from_str(json).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].currencies.len(), 1);
        assert_eq!(items[0].currencies[0].rate, 2.85);
        assert_eq!(items[0].currencies[0].quantity, 1.0);
    }

    #[test]
    fn process_response_builds_rate() {
        let items = vec![NbgResponse {
            date: "2026-03-18T00:00:00".to_string(),
            currencies: vec![NbgCurrency { quantity: 1.0, rate: 2.85 }],
        }];
        let rate = process_response(&items, Currency::USD).unwrap();
        assert_eq!(rate.country, Country::Georgia);
        assert_eq!(rate.currency, Currency::USD);
        assert!((rate.rate - 2.85).abs() < 1e-10);
        assert_eq!(rate.date, "2026-03-18");
    }

    #[test]
    fn process_response_divides_by_quantity() {
        let items = vec![NbgResponse {
            date: "2026-03-18T00:00:00".to_string(),
            currencies: vec![NbgCurrency { quantity: 100.0, rate: 285.0 }],
        }];
        let rate = process_response(&items, Currency::EUR).unwrap();
        assert!((rate.rate - 2.85).abs() < 1e-10);
    }

    #[test]
    fn process_response_empty_returns_none() {
        assert!(process_response(&[], Currency::USD).is_none());
    }

    #[test]
    fn process_response_no_currencies_returns_none() {
        let items = vec![NbgResponse {
            date: "2026-03-18T00:00:00".to_string(),
            currencies: vec![],
        }];
        assert!(process_response(&items, Currency::USD).is_none());
    }

    #[tokio::test]
    async fn fetch_from_with_mock_server() {
        use wiremock::{MockServer, Mock, ResponseTemplate};
        use wiremock::matchers::{method, query_param};

        let json = r#"[{"date":"2026-03-18T00:00:00","currencies":[{"code":"USD","quantity":1.0,"rate":2.85}]}]"#;

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(query_param("currencies", "USD"))
            .respond_with(ResponseTemplate::new(200).set_body_string(json))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let rates = fetch(&client, &server.uri(), &[Currency::USD]).await.unwrap();
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].currency, Currency::USD);
        assert_eq!(rates[0].country, Country::Georgia);
        assert!((rates[0].rate - 2.85).abs() < 1e-10);
        assert_eq!(rates[0].date, "2026-03-18");
    }
}
