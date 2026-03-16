use crate::display;
use crate::models::ExchangeRate;
use crate::OutputFormat;

pub fn print_rates(rates: &[ExchangeRate], format: OutputFormat) {
    match format {
        OutputFormat::Table => display::print_rates(rates),
        OutputFormat::Json => {
            // Panics on NaN/Infinity — acceptable, indicates broken upstream data
            println!("{}", format_json(rates));
        }
    }
}

pub(crate) fn format_json(rates: &[ExchangeRate]) -> String {
    // Panics if any rate is NaN or Infinity (broken upstream data)
    serde_json::to_string_pretty(rates).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::models::{Country, Currency, ExchangeRate};
    use super::*;

    fn make_rates() -> Vec<ExchangeRate> {
        vec![
            ExchangeRate {
                country: Country::Poland,
                currency: Currency::USD,
                rate: 4.0123,
                date: "2026-03-16".to_string(),
            },
            ExchangeRate {
                country: Country::Belarus,
                currency: Currency::EUR,
                rate: 3.1200,
                date: "2026-03-16".to_string(),
            },
        ]
    }

    #[test]
    fn format_json_produces_pretty_json_array() {
        let rates = make_rates();
        let json = format_json(&rates);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["country"], "poland");
        assert_eq!(parsed[0]["currency"], "usd");
        assert_eq!(parsed[1]["country"], "belarus");
        assert_eq!(parsed[1]["currency"], "eur");
        assert!(json.contains('\n'), "output should be pretty-printed");
        assert_eq!(parsed[0]["rate"], 4.0123);
        assert_eq!(parsed[0]["date"], "2026-03-16");
    }

    #[test]
    fn format_json_empty_slice_produces_empty_array() {
        let json = format_json(&[]);
        assert_eq!(json.trim(), "[]");
    }
}
