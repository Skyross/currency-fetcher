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
    Ok(s.replace('\u{00A0}', "")
        .replace(' ', "")
        .replace(',', ".")
        .parse()?)
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

pub async fn fetch(client: &Client, currencies: &[Currency]) -> anyhow::Result<Vec<ExchangeRate>> {
    let url = "https://www.cbr.ru/scripts/XML_daily.asp";
    let text = client.get(url).send().await?.text().await?;
    let val_curs: ValCurs = quick_xml::de::from_str(&text)?;
    let date = convert_date(&val_curs.date);

    let wanted: HashSet<&str> = currencies.iter().map(|c| match c {
        Currency::USD => "USD",
        Currency::EUR => "EUR",
        Currency::GBP => "GBP",
    }).collect();

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
        let nominal: u32 = v.nominal.trim().parse()
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
    fn parse_nominal_exact() {
        // nominal "10" should divide value "876,3250" to exactly 87.6325
        let nominal: u32 = "10".trim().parse().unwrap();
        let value = parse_cbr_decimal("876,3250").unwrap();
        let result = value / f64::from(nominal);
        assert!((result - 87.6325).abs() < 1e-10);
    }
}
