mod cbr;
mod nbg;
mod nbp;
mod nbrb;
mod util;

use crate::models::{Country, Currency, ExchangeRate};
use reqwest::Client;

pub async fn fetch_rates(
    client: &Client,
    country: Country,
    currencies: &[Currency],
) -> anyhow::Result<Vec<ExchangeRate>> {
    match country {
        Country::Belarus => nbrb::fetch(client, currencies).await,
        Country::Georgia => nbg::fetch(client, currencies).await,
        Country::Poland => nbp::fetch(client, currencies).await,
        Country::Russia => cbr::fetch(client, currencies).await,
    }
}
