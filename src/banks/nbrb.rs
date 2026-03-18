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

fn process_response(resp: &NbrbResponse, cur: Currency) -> ExchangeRate {
    ExchangeRate {
        country: Country::Belarus,
        currency: cur,
        rate: resp.official_rate / resp.scale,
        date: util::trim_date(&resp.date),
    }
}

pub(super) async fn fetch(client: &Client, base_url: &str, currencies: &[Currency]) -> anyhow::Result<Vec<ExchangeRate>> {
    let mut rates = Vec::new();
    for &cur in currencies {
        let url = format!("{}/{}?parammode=2", base_url, cur);
        let resp: NbrbResponse = client.get(&url).send().await?.json().await?;
        rates.push(process_response(&resp, cur));
    }
    Ok(rates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nbrb_response_deserializes() {
        let json = r#"{"Cur_ID":431,"Date":"2026-03-18T00:00:00","Cur_Abbreviation":"USD","Cur_Scale":1,"Cur_Name":"Доллар США","Cur_OfficialRate":3.2800}"#;
        let resp: NbrbResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.official_rate, 3.28);
        assert_eq!(resp.scale, 1.0);
        assert!(resp.date.starts_with("2026-03-18"));
    }

    #[test]
    fn process_response_builds_rate() {
        let resp = NbrbResponse {
            official_rate: 3.28,
            scale: 1.0,
            date: "2026-03-18T00:00:00".to_string(),
        };
        let rate = process_response(&resp, Currency::USD);
        assert_eq!(rate.country, Country::Belarus);
        assert_eq!(rate.currency, Currency::USD);
        assert_eq!(rate.rate, 3.28);
        assert_eq!(rate.date, "2026-03-18");
    }

    #[test]
    fn process_response_divides_by_scale() {
        let resp = NbrbResponse {
            official_rate: 4.56,
            scale: 100.0,
            date: "2026-03-18T00:00:00".to_string(),
        };
        let rate = process_response(&resp, Currency::EUR);
        assert!((rate.rate - 0.0456).abs() < 1e-10);
    }

    #[tokio::test]
    async fn fetch_from_with_mock_server() {
        use wiremock::{MockServer, Mock, ResponseTemplate};
        use wiremock::matchers::{method, path};

        let json = r#"{"Cur_ID":431,"Date":"2026-03-18T00:00:00","Cur_Abbreviation":"USD","Cur_Scale":1,"Cur_Name":"US Dollar","Cur_OfficialRate":3.2800}"#;

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/USD"))
            .respond_with(ResponseTemplate::new(200).set_body_string(json))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let rates = fetch(&client, &server.uri(), &[Currency::USD]).await.unwrap();
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].currency, Currency::USD);
        assert_eq!(rates[0].country, Country::Belarus);
        assert_eq!(rates[0].rate, 3.28);
        assert_eq!(rates[0].date, "2026-03-18");
    }
}
