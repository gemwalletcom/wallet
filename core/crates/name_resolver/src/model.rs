use idna::uts46::{AsciiDenyList, DnsLength, Hyphens, Uts46};

pub struct NameQuery {
    pub name: String,
    pub domain: String,
    pub suffix: String,
}

impl NameQuery {
    pub fn new(domain: &str) -> Self {
        let name = domain.split('.').next().unwrap_or(domain).to_string();
        let suffix = domain.get(name.len() + 1..).unwrap_or_default().to_string();
        Self {
            name,
            domain: domain.to_string(),
            suffix,
        }
    }

    pub fn ascii_domain(&self) -> Result<String, idna::Errors> {
        let domain = Uts46::new().to_ascii(self.domain.as_bytes(), AsciiDenyList::STD3, Hyphens::Allow, DnsLength::Ignore)?;
        Ok(domain.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::NameQuery;

    #[test]
    fn test_name_query_single_suffix() {
        let query = NameQuery::new("alice.eth");

        assert_eq!(query.name, "alice");
        assert_eq!(query.domain, "alice.eth");
        assert_eq!(query.suffix, "eth");
    }

    #[test]
    fn test_name_query_multi_suffix() {
        let query = NameQuery::new("alice.base.eth");

        assert_eq!(query.name, "alice");
        assert_eq!(query.domain, "alice.base.eth");
        assert_eq!(query.suffix, "base.eth");
    }

    #[test]
    fn test_name_query_no_dot() {
        let query = NameQuery::new("alice");

        assert_eq!(query.name, "alice");
        assert_eq!(query.domain, "alice");
        assert_eq!(query.suffix, "");
    }

    #[test]
    fn test_ascii_domain() {
        assert_eq!(NameQuery::new("Vitalik.ETH").ascii_domain().unwrap(), "vitalik.eth");
        assert_eq!(NameQuery::new("🐈.hl").ascii_domain().unwrap(), "xn--zn8h.hl");
    }
}
