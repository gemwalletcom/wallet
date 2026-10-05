#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemSearchScope {
    All,
    List { id: String },
}

#[uniffi::export]
impl GemSearchScope {
    pub fn search_key(&self, query: String) -> String {
        let query = query.trim();
        match self {
            Self::List { id } if query.is_empty() => format!("tag:{id}"),
            Self::All | Self::List { .. } => query.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GemSearchScope;

    #[test]
    fn test_search_key() {
        let list = GemSearchScope::List { id: "stocks".to_string() };
        assert_eq!(GemSearchScope::All.search_key(" btc ".to_string()), "btc");
        assert_eq!(GemSearchScope::All.search_key(String::new()), "");
        assert_eq!(list.search_key(String::new()), "tag:stocks");
        assert_eq!(list.search_key(" eth ".to_string()), "eth");
    }
}
