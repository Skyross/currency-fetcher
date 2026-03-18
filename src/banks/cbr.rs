use crate::models::{Country, Currency, ExchangeRate};
use anyhow::Context;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
#[serde(rename = "ValCurs")]
struct ValCurs {
    #[serde(rename = "@Date")]
    date: String,
    #[serde(rename = "Valute", default)]
    valutes: Vec<Valute>,
}

#[derive(Debug, Deserialize)]
struct Valute {
    #[serde(rename = "CharCode")]
    char_code: String,
    #[serde(rename = "Nominal")]
    nominal: String,
    #[serde(rename = "Value")]
    value: String,
}

fn parse_cbr_decimal(s: &str) -> anyhow::Result<f64> {
    Ok(s.replace(['\u{00A0}', ' '], "").replace(',', ".").parse()?)
}

fn convert_date(dd_mm_yyyy: &str) -> String {
    // "15.01.2024" -> "2024-01-15"
    let parts: Vec<&str> = dd_mm_yyyy.split('.').collect();
    if parts.len() == 3 {
        format!("{}-{}-{}", parts[2], parts[1], parts[0])
    } else {
        dd_mm_yyyy.to_string()
    }
}

fn process_xml(text: &str, currencies: &[Currency]) -> anyhow::Result<Vec<ExchangeRate>> {
    let val_curs: ValCurs = quick_xml::de::from_str(text)?;
    let date = convert_date(&val_curs.date);

    let wanted: HashSet<&str> = currencies
        .iter()
        .map(|c| match c {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
        })
        .collect();

    let mut rates = Vec::new();
    for v in &val_curs.valutes {
        if !wanted.contains(&v.char_code.as_str()) {
            continue;
        }
        let currency = match v.char_code.as_str() {
            "USD" => Currency::USD,
            "EUR" => Currency::EUR,
            "GBP" => Currency::GBP,
            _ => continue,
        };
        let nominal: u32 = v
            .nominal
            .trim()
            .parse()
            .with_context(|| format!("CBR: invalid nominal '{}'", v.nominal))?;
        let value = parse_cbr_decimal(&v.value)?;
        rates.push(ExchangeRate {
            country: Country::Russia,
            currency,
            rate: value / f64::from(nominal),
            date: date.clone(),
        });
    }
    Ok(rates)
}

pub(super) async fn fetch(
    client: &Client,
    url: &str,
    currencies: &[Currency],
) -> anyhow::Result<Vec<ExchangeRate>> {
    let text = client.get(url).send().await?.text().await?;
    process_xml(&text, currencies)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cbr_decimal_handles_thousands_separator() {
        assert_eq!(parse_cbr_decimal("1\u{00A0}234,56").unwrap(), 1234.56);
        assert_eq!(parse_cbr_decimal("1 234,56").unwrap(), 1234.56);
        assert_eq!(parse_cbr_decimal("87,6325").unwrap(), 87.6325);
    }

    #[test]
    fn parse_cbr_decimal_plain_integer() {
        assert_eq!(parse_cbr_decimal("100").unwrap(), 100.0);
    }

    #[test]
    fn parse_cbr_decimal_invalid() {
        assert!(parse_cbr_decimal("abc").is_err());
    }

    #[test]
    fn parse_nominal_exact() {
        let nominal: u32 = "10".trim().parse().unwrap();
        let value = parse_cbr_decimal("876,3250").unwrap();
        let result = value / f64::from(nominal);
        assert!((result - 87.6325).abs() < 1e-10);
    }

    #[test]
    fn convert_date_valid() {
        assert_eq!(convert_date("15.01.2024"), "2024-01-15");
        assert_eq!(convert_date("01.12.2026"), "2026-12-01");
    }

    #[test]
    fn convert_date_invalid_passes_through() {
        assert_eq!(convert_date("2024-01-15"), "2024-01-15");
        assert_eq!(convert_date("bogus"), "bogus");
    }

    #[test]
    fn valcurs_deserializes_from_xml() {
        let xml = r#"<?xml version="1.0" encoding="windows-1251"?>
<ValCurs Date="18.03.2026" name="Foreign Currency Market">
    <Valute ID="R01235">
        <CharCode>USD</CharCode>
        <Nominal>1</Nominal>
        <Value>87,6325</Value>
    </Valute>
    <Valute ID="R01239">
        <CharCode>EUR</CharCode>
        <Nominal>1</Nominal>
        <Value>95,1234</Value>
    </Valute>
</ValCurs>"#;
        let val_curs: ValCurs = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(val_curs.date, "18.03.2026");
        assert_eq!(val_curs.valutes.len(), 2);
        assert_eq!(val_curs.valutes[0].char_code, "USD");
        assert_eq!(val_curs.valutes[0].nominal, "1");
        assert_eq!(val_curs.valutes[0].value, "87,6325");
        assert_eq!(val_curs.valutes[1].char_code, "EUR");
    }

    #[test]
    fn valcurs_empty_valutes() {
        let xml = r#"<ValCurs Date="18.03.2026"></ValCurs>"#;
        let val_curs: ValCurs = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(val_curs.date, "18.03.2026");
        assert!(val_curs.valutes.is_empty());
    }

    #[test]
    fn process_xml_extracts_requested_currencies() {
        let xml = r#"<?xml version="1.0" encoding="windows-1251"?>
<ValCurs Date="18.03.2026" name="Foreign Currency Market">
    <Valute ID="R01235">
        <CharCode>USD</CharCode>
        <Nominal>1</Nominal>
        <Value>87,6325</Value>
    </Valute>
    <Valute ID="R01239">
        <CharCode>EUR</CharCode>
        <Nominal>1</Nominal>
        <Value>95,1234</Value>
    </Valute>
    <Valute ID="R01270">
        <CharCode>CNY</CharCode>
        <Nominal>10</Nominal>
        <Value>121,4500</Value>
    </Valute>
</ValCurs>"#;
        let rates = process_xml(xml, &[Currency::USD, Currency::EUR]).unwrap();
        assert_eq!(rates.len(), 2);
        assert_eq!(rates[0].currency, Currency::USD);
        assert_eq!(rates[0].country, Country::Russia);
        assert!((rates[0].rate - 87.6325).abs() < 1e-10);
        assert_eq!(rates[0].date, "2026-03-18");
        assert_eq!(rates[1].currency, Currency::EUR);
        assert!((rates[1].rate - 95.1234).abs() < 1e-10);
    }

    #[test]
    fn process_xml_filters_unwanted_currencies() {
        let xml = r#"<ValCurs Date="18.03.2026">
    <Valute ID="R01235">
        <CharCode>USD</CharCode>
        <Nominal>1</Nominal>
        <Value>87,6325</Value>
    </Valute>
    <Valute ID="R01239">
        <CharCode>EUR</CharCode>
        <Nominal>1</Nominal>
        <Value>95,1234</Value>
    </Valute>
</ValCurs>"#;
        let rates = process_xml(xml, &[Currency::GBP]).unwrap();
        assert!(rates.is_empty());
    }

    #[test]
    fn process_xml_handles_nominal_division() {
        let xml = r#"<ValCurs Date="18.03.2026">
    <Valute>
        <CharCode>GBP</CharCode>
        <Nominal>10</Nominal>
        <Value>1100,5000</Value>
    </Valute>
</ValCurs>"#;
        let rates = process_xml(xml, &[Currency::GBP]).unwrap();
        assert_eq!(rates.len(), 1);
        assert!((rates[0].rate - 110.05).abs() < 1e-10);
    }

    #[test]
    fn process_xml_empty_response() {
        let xml = r#"<ValCurs Date="18.03.2026"></ValCurs>"#;
        let rates = process_xml(xml, &[Currency::USD]).unwrap();
        assert!(rates.is_empty());
    }

    #[tokio::test]
    async fn fetch_from_with_mock_server() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let xml = r#"<ValCurs Date="18.03.2026">
    <Valute><CharCode>USD</CharCode><Nominal>1</Nominal><Value>87,6325</Value></Valute>
    <Valute><CharCode>EUR</CharCode><Nominal>1</Nominal><Value>95,1234</Value></Valute>
</ValCurs>"#;

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xml))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let rates = fetch(&client, &server.uri(), &[Currency::USD, Currency::EUR])
            .await
            .unwrap();
        assert_eq!(rates.len(), 2);
        assert_eq!(rates[0].currency, Currency::USD);
        assert!((rates[0].rate - 87.6325).abs() < 1e-10);
        assert_eq!(rates[1].currency, Currency::EUR);
    }
}
