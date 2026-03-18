mod cbr;
mod nbg;
mod nbp;
mod nbrb;
mod util;

use crate::models::{Country, Currency, ExchangeRate};
use reqwest::Client;

fn base_url(country: Country) -> &'static str {
    match country {
        Country::Belarus => "https://api.nbrb.by/exrates/rates",
        Country::Georgia => "https://nbg.gov.ge/gw/api/ct/monetarypolicy/currencies/en/json/",
        Country::Poland => "https://api.nbp.pl/api/exchangerates/rates/a",
        Country::Russia => "https://www.cbr.ru/scripts/XML_daily.asp",
    }
}

pub(crate) async fn fetch_rates_from(
    client: &Client,
    country: Country,
    url: &str,
    currencies: &[Currency],
) -> anyhow::Result<Vec<ExchangeRate>> {
    match country {
        Country::Belarus => nbrb::fetch(client, url, currencies).await,
        Country::Georgia => nbg::fetch(client, url, currencies).await,
        Country::Poland => nbp::fetch(client, url, currencies).await,
        Country::Russia => cbr::fetch(client, url, currencies).await,
    }
}

pub async fn fetch_rates(
    client: &Client,
    country: Country,
    currencies: &[Currency],
) -> anyhow::Result<Vec<ExchangeRate>> {
    fetch_rates_from(client, country, base_url(country), currencies).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn base_url_returns_correct_urls() {
        assert!(base_url(Country::Belarus).contains("nbrb.by"));
        assert!(base_url(Country::Georgia).contains("nbg.gov.ge"));
        assert!(base_url(Country::Poland).contains("nbp.pl"));
        assert!(base_url(Country::Russia).contains("cbr.ru"));
    }

    #[tokio::test]
    async fn fetch_rates_from_dispatches_to_cbr() {
        let xml = r#"<ValCurs Date="18.03.2026">
    <Valute><CharCode>USD</CharCode><Nominal>1</Nominal><Value>87,6325</Value></Valute>
</ValCurs>"#;

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xml))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let rates = fetch_rates_from(&client, Country::Russia, &server.uri(), &[Currency::USD])
            .await
            .unwrap();
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].country, Country::Russia);
    }

    #[tokio::test]
    async fn fetch_rates_from_dispatches_to_nbrb() {
        let json =
            r#"{"Cur_ID":431,"Date":"2026-03-18T00:00:00","Cur_Scale":1,"Cur_OfficialRate":3.28}"#;

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(json))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let rates = fetch_rates_from(&client, Country::Belarus, &server.uri(), &[Currency::USD])
            .await
            .unwrap();
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].country, Country::Belarus);
    }

    #[tokio::test]
    async fn fetch_rates_from_dispatches_to_nbg() {
        let json = r#"[{"date":"2026-03-18T00:00:00","currencies":[{"code":"USD","quantity":1.0,"rate":2.85}]}]"#;

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(json))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let rates = fetch_rates_from(&client, Country::Georgia, &server.uri(), &[Currency::USD])
            .await
            .unwrap();
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].country, Country::Georgia);
    }

    #[tokio::test]
    async fn fetch_rates_from_dispatches_to_nbp() {
        let json = r#"{"rates":[{"effectiveDate":"2026-03-17","mid":4.0123}]}"#;

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(json))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let rates = fetch_rates_from(&client, Country::Poland, &server.uri(), &[Currency::USD])
            .await
            .unwrap();
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].country, Country::Poland);
    }
}
