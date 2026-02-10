use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    USD,
    EUR,
    GBP,
}

impl Currency {
    pub fn all() -> &'static [Currency] {
        &[Currency::USD, Currency::EUR, Currency::GBP]
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::USD => write!(f, "USD"),
            Currency::EUR => write!(f, "EUR"),
            Currency::GBP => write!(f, "GBP"),
        }
    }
}

impl std::str::FromStr for Currency {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD" => Ok(Currency::USD),
            "EUR" => Ok(Currency::EUR),
            "GBP" => Ok(Currency::GBP),
            other => anyhow::bail!("unknown currency: {other}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Country {
    Belarus,
    Georgia,
    Poland,
    Russia,
}

impl Country {
    pub fn all() -> &'static [Country] {
        &[Country::Belarus, Country::Georgia, Country::Poland, Country::Russia]
    }
}

impl fmt::Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Country::Belarus => write!(f, "Belarus"),
            Country::Georgia => write!(f, "Georgia"),
            Country::Poland => write!(f, "Poland"),
            Country::Russia => write!(f, "Russia"),
        }
    }
}

impl std::str::FromStr for Country {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "belarus" | "by" => Ok(Country::Belarus),
            "georgia" | "ge" => Ok(Country::Georgia),
            "poland" | "pl" => Ok(Country::Poland),
            "russia" | "ru" => Ok(Country::Russia),
            other => anyhow::bail!("unknown country: {other}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExchangeRate {
    pub country: Country,
    pub currency: Currency,
    pub rate: f64,
    pub date: String,
}
