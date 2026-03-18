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

fn process_response(resp: &NbpResponse, cur: Currency) -> Option<ExchangeRate> {
    let r = resp.rates.first()?;
    Some(ExchangeRate {
        country: Country::Poland,
        currency: cur,
        rate: r.mid,
        date: r.effective_date.clone(),
    })
}

pub(super) async fn fetch(client: &Client, base_url: &str, currencies: &[Currency]) -> anyhow::Result<Vec<ExchangeRate>> {
    let mut rates = Vec::new();
    for &cur in currencies {
        let url = format!("{}/{}/?format=json", base_url, cur.as_lower_code());
        let resp: NbpResponse = client.get(&url).send().await?.json().await?;
        if let Some(rate) = process_response(&resp, cur) {
            rates.push(rate);
        }
    }
    Ok(rates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nbp_response_deserializes() {
        let json = r#"{"table":"A","currency":"US Dollar","code":"USD","rates":[{"no":"053/A/NBP/2026","effectiveDate":"2026-03-17","mid":4.0123}]}"#;
        let resp: NbpResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.rates.len(), 1);
        assert_eq!(resp.rates[0].effective_date, "2026-03-17");
        assert_eq!(resp.rates[0].mid, 4.0123);
    }

    #[test]
    fn process_response_builds_rate() {
        let resp = NbpResponse {
            rates: vec![NbpRate {
                effective_date: "2026-03-17".to_string(),
                mid: 4.0123,
            }],
        };
        let rate = process_response(&resp, Currency::USD).unwrap();
        assert_eq!(rate.country, Country::Poland);
        assert_eq!(rate.currency, Currency::USD);
        assert_eq!(rate.rate, 4.0123);
        assert_eq!(rate.date, "2026-03-17");
    }

    #[test]
    fn process_response_empty_rates_returns_none() {
        let resp = NbpResponse { rates: vec![] };
        assert!(process_response(&resp, Currency::EUR).is_none());
    }

    #[tokio::test]
    async fn fetch_from_with_mock_server() {
        use wiremock::{MockServer, Mock, ResponseTemplate};
        use wiremock::matchers::{method, path};

        let json = r#"{"table":"A","currency":"US Dollar","code":"USD","rates":[{"no":"053","effectiveDate":"2026-03-17","mid":4.0123}]}"#;

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/usd/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(json))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let rates = fetch(&client, &server.uri(), &[Currency::USD]).await.unwrap();
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].currency, Currency::USD);
        assert_eq!(rates[0].country, Country::Poland);
        assert_eq!(rates[0].rate, 4.0123);
        assert_eq!(rates[0].date, "2026-03-17");
    }
}
