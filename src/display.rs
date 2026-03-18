use crate::models::ExchangeRate;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct RateRow {
    #[tabled(rename = "Country")]
    country: String,
    #[tabled(rename = "Currency")]
    currency: String,
    #[tabled(rename = "Rate")]
    rate: String,
    #[tabled(rename = "Date")]
    date: String,
}

pub(crate) fn print_rates(rates: &[ExchangeRate]) {
    if rates.is_empty() {
        println!("No rates fetched.");
        return;
    }

    let rows: Vec<RateRow> = rates
        .iter()
        .map(|r| RateRow {
            country: r.country.to_string(),
            currency: r.currency.to_string(),
            rate: format!("{:.4}", r.rate),
            date: r.date.clone(),
        })
        .collect();

    println!("{}", Table::new(rows));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Country, Currency};

    #[test]
    fn print_rates_with_data_does_not_panic() {
        let rates = vec![ExchangeRate {
            country: Country::Poland,
            currency: Currency::USD,
            rate: 4.0123,
            date: "2026-03-16".to_string(),
        }];
        // Exercises the table-building logic; output goes to stdout
        print_rates(&rates);
    }

    #[test]
    fn print_rates_empty_does_not_panic() {
        print_rates(&[]);
    }
}
