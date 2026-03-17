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
