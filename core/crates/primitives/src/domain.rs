use url::Url;

pub fn parse_domain(domain: &str) -> Option<Url> {
    if domain.is_empty() || domain.contains(['/', '?', '#', '@', '\\']) || domain.chars().any(char::is_whitespace) {
        return None;
    }
    let url = Url::parse(&format!("https://{domain}")).ok()?;
    url.host_str()?;
    Some(url)
}

#[cfg(test)]
mod tests {
    use super::parse_domain;

    #[test]
    fn test_parse_domain() {
        for (domain, host, port) in [
            ("example.com", "example.com", None),
            ("EXAMPLE.com:8443", "example.com", Some(8443)),
            ("localhost:3000", "localhost", Some(3000)),
            ("[::1]:8443", "[::1]", Some(8443)),
        ] {
            let url = parse_domain(domain).unwrap();
            assert_eq!(url.host_str(), Some(host));
            assert_eq!(url.port(), port);
        }

        for domain in [
            "",
            "https://example.com",
            "example.com/path",
            "example.com?query",
            "example.com#fragment",
            "user@example.com",
            "example.com\\path",
            " example.com",
            "example.com\n",
            "example.com:invalid",
            "example.com:65536",
        ] {
            assert_eq!(parse_domain(domain), None, "{domain}");
        }
    }
}
