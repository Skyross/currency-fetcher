use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::upper_case_acronyms)]
pub enum Currency {
    USD,
    EUR,
    GBP,
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

impl Currency {
    pub(crate) fn as_lower_code(self) -> &'static str {
        match self {
            Currency::USD => "usd",
            Currency::EUR => "eur",
            Currency::GBP => "gbp",
        }
    }
}

impl std::str::FromStr for Currency {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("usd") {
            Ok(Currency::USD)
        } else if s.eq_ignore_ascii_case("eur") {
            Ok(Currency::EUR)
        } else if s.eq_ignore_ascii_case("gbp") {
            Ok(Currency::GBP)
        } else {
            anyhow::bail!("unknown currency: {s}")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Country {
    Belarus,
    Georgia,
    Poland,
    Russia,
}

impl Country {
    pub fn all() -> &'static [Country] {
        &[
            Country::Belarus,
            Country::Georgia,
            Country::Poland,
            Country::Russia,
        ]
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

#[derive(Debug, Clone, Serialize)]
pub struct ExchangeRate {
    pub country: Country,
    pub currency: Currency,
    pub rate: f64,
    pub date: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exchange_rate_serializes_to_flat_json_shape() {
        let rate = ExchangeRate {
            country: Country::Poland,
            currency: Currency::USD,
            rate: 4.0123,
            date: "2026-03-16".to_string(),
        };
        let json = serde_json::to_string(&rate).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["country"], "poland");
        assert_eq!(v["currency"], "usd");
        assert_eq!(v["rate"], 4.0123);
        assert_eq!(v["date"], "2026-03-16");
    }

    #[test]
    fn country_and_currency_sort_by_declaration_order() {
        assert!(Country::Belarus < Country::Georgia);
        assert!(Country::Georgia < Country::Poland);
        assert!(Country::Poland < Country::Russia);
        assert!(Currency::USD < Currency::EUR);
        assert!(Currency::EUR < Currency::GBP);
    }

    #[test]
    fn currency_display() {
        assert_eq!(Currency::USD.to_string(), "USD");
        assert_eq!(Currency::EUR.to_string(), "EUR");
        assert_eq!(Currency::GBP.to_string(), "GBP");
    }

    #[test]
    fn currency_from_str_case_insensitive() {
        assert_eq!("usd".parse::<Currency>().unwrap(), Currency::USD);
        assert_eq!("USD".parse::<Currency>().unwrap(), Currency::USD);
        assert_eq!("Eur".parse::<Currency>().unwrap(), Currency::EUR);
        assert_eq!("gbp".parse::<Currency>().unwrap(), Currency::GBP);
    }

    #[test]
    fn currency_from_str_unknown_errors() {
        let err = "xyz".parse::<Currency>();
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("unknown currency"));
    }

    #[test]
    fn currency_as_lower_code() {
        assert_eq!(Currency::USD.as_lower_code(), "usd");
        assert_eq!(Currency::EUR.as_lower_code(), "eur");
        assert_eq!(Currency::GBP.as_lower_code(), "gbp");
    }

    #[test]
    fn country_display() {
        assert_eq!(Country::Belarus.to_string(), "Belarus");
        assert_eq!(Country::Georgia.to_string(), "Georgia");
        assert_eq!(Country::Poland.to_string(), "Poland");
        assert_eq!(Country::Russia.to_string(), "Russia");
    }

    #[test]
    fn country_from_str_full_names() {
        assert_eq!("belarus".parse::<Country>().unwrap(), Country::Belarus);
        assert_eq!("GEORGIA".parse::<Country>().unwrap(), Country::Georgia);
        assert_eq!("Poland".parse::<Country>().unwrap(), Country::Poland);
        assert_eq!("russia".parse::<Country>().unwrap(), Country::Russia);
    }

    #[test]
    fn country_from_str_abbreviations() {
        assert_eq!("by".parse::<Country>().unwrap(), Country::Belarus);
        assert_eq!("ge".parse::<Country>().unwrap(), Country::Georgia);
        assert_eq!("pl".parse::<Country>().unwrap(), Country::Poland);
        assert_eq!("ru".parse::<Country>().unwrap(), Country::Russia);
    }

    #[test]
    fn country_from_str_unknown_errors() {
        let err = "mars".parse::<Country>();
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("unknown country"));
    }

    #[test]
    fn country_all_returns_all_variants() {
        let all = Country::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&Country::Belarus));
        assert!(all.contains(&Country::Georgia));
        assert!(all.contains(&Country::Poland));
        assert!(all.contains(&Country::Russia));
    }

    #[test]
    fn country_serializes_lowercase() {
        let json = serde_json::to_string(&Country::Belarus).unwrap();
        assert_eq!(json, "\"belarus\"");
    }

    #[test]
    fn currency_serializes_lowercase() {
        let json = serde_json::to_string(&Currency::USD).unwrap();
        assert_eq!(json, "\"usd\"");
    }
}
