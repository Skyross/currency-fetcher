/// Trims an ISO 8601 datetime string to just the date part.
/// "2024-01-15T00:00:00" → "2024-01-15"
/// Already-date strings are returned unchanged.
pub(super) fn trim_date(s: &str) -> String {
    s.split('T').next().unwrap_or(s).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_date_strips_time_component() {
        assert_eq!(trim_date("2024-01-15T00:00:00"), "2024-01-15");
    }

    #[test]
    fn trim_date_returns_plain_date_unchanged() {
        assert_eq!(trim_date("2024-01-15"), "2024-01-15");
    }
}
