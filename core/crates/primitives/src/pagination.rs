pub const MAX_QUERY_LIMIT: usize = 100;
pub const TRANSACTIONS_LIMIT: usize = 1000;

pub fn transactions_page_limit(fetched: usize) -> Option<usize> {
    let remaining = TRANSACTIONS_LIMIT.saturating_sub(fetched);
    (remaining > 0).then_some(remaining.min(MAX_QUERY_LIMIT))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transactions_page_limit_stops_at_the_transactions_limit() {
        assert_eq!(transactions_page_limit(0), Some(MAX_QUERY_LIMIT));
        assert_eq!(transactions_page_limit(TRANSACTIONS_LIMIT - 30), Some(30));
        assert_eq!(transactions_page_limit(TRANSACTIONS_LIMIT), None);
        assert_eq!(transactions_page_limit(TRANSACTIONS_LIMIT + 1), None);
    }
}
