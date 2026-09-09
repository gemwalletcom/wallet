use std::str;

use super::SiwsMessage;

const PREAMBLE_SUFFIX: &str = " wants you to sign in with your Solana account:";
const FIELD_NAMES: [&str; 8] = ["URI", "Version", "Chain ID", "Nonce", "Issued At", "Expiration Time", "Not Before", "Request ID"];

impl SiwsMessage {
    pub fn decode_base58(data: &str) -> Result<Option<Self>, String> {
        let bytes = bs58::decode(data).into_vec().map_err(|_| "Invalid Base58 message")?;
        match str::from_utf8(&bytes) {
            Ok(text) => Self::parse(text),
            Err(_) => Ok(None),
        }
    }

    pub fn parse(raw: &str) -> Result<Option<Self>, String> {
        let mut lines = raw.splitn(2, '\n');
        let Some(domain) = lines.next().and_then(|header| header.strip_suffix(PREAMBLE_SUFFIX)) else {
            return Ok(None);
        };
        let body = lines.next().ok_or("Invalid SIWS header")?;
        let (address, body) = match body.split_once('\n') {
            Some((address, body)) => (address, Some(body.strip_prefix('\n').ok_or("Invalid SIWS separator")?)),
            None => (body, None),
        };
        let (statement, fields) = body.map(Self::parse_statement).transpose()?.unwrap_or((None, None));
        let mut lines = fields.into_iter().flat_map(|fields| fields.split('\n')).peekable();
        let [uri, version, chain_id, nonce, issued_at, expiration_time, not_before, request_id] = FIELD_NAMES.map(|name| {
            let (label, value) = lines.peek()?.split_once(": ")?;
            if label != name {
                return None;
            }
            let value = value.to_string();
            lines.next();
            Some(value)
        });
        let resources = match lines.next() {
            None => vec![],
            Some("Resources:") => lines
                .map(|line| line.strip_prefix("- ").map(str::to_string).ok_or("Invalid SIWS resource".to_string()))
                .collect::<Result<_, _>>()?,
            Some(_) => return Err("Invalid SIWS field order or trailing content".to_string()),
        };
        Ok(Some(Self {
            domain: domain.to_string(),
            address: address.to_string(),
            statement,
            uri,
            version,
            chain_id,
            nonce,
            issued_at,
            expiration_time,
            not_before,
            request_id,
            resources,
        }))
    }

    fn parse_statement(body: &str) -> Result<(Option<String>, Option<&str>), String> {
        let label = body.split_once(':').map(|(label, _)| label);
        if label.is_some_and(|label| FIELD_NAMES.contains(&label) || label == "Resources") {
            return Ok((None, Some(body)));
        }
        let (statement, fields) = match body.split_once("\n\n") {
            Some((statement, fields)) => (statement, Some(fields)),
            None => (body, None),
        };
        if statement.is_empty() || !statement.is_ascii() || statement.chars().any(char::is_control) {
            return Err("Invalid SIWS statement".to_string());
        }
        Ok((Some(statement.to_string()), fields))
    }
}

#[cfg(test)]
mod tests {
    use super::SiwsMessage;
    use crate::testkit::mock_siws_message;

    #[test]
    fn test_decode_base58() {
        let message = include_str!("../../testdata/siws_complete.txt");
        assert_eq!(SiwsMessage::decode_base58(&bs58::encode(message).into_string()), Ok(Some(SiwsMessage::mock_complete())));

        for bytes in [b"ordinary message".as_slice(), &[0xff, 0x00]] {
            assert_eq!(SiwsMessage::decode_base58(&bs58::encode(bytes).into_string()), Ok(None));
        }

        let malformed = "example.com wants you to sign in with your Solana account:";
        assert_eq!(SiwsMessage::decode_base58(&bs58::encode(malformed).into_string()), Err("Invalid SIWS header".to_string()));
        assert_eq!(SiwsMessage::decode_base58("!"), Err("Invalid Base58 message".to_string()));
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            SiwsMessage::mock_complete(),
            SiwsMessage {
                domain: "example.com".to_string(),
                address: "AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9".to_string(),
                statement: Some("Sign in to the app.".to_string()),
                uri: Some("https://example.com/login".to_string()),
                version: Some("1".to_string()),
                chain_id: Some("mainnet".to_string()),
                nonce: Some("8hK9pX32".to_string()),
                issued_at: Some("2026-09-01T12:00:00Z".to_string()),
                expiration_time: Some("2026-09-02T12:00:00Z".to_string()),
                not_before: Some("2026-09-01T11:00:00Z".to_string()),
                request_id: Some("request-123".to_string()),
                resources: vec!["https://example.com/terms".to_string(), "https://example.com/privacy".to_string()],
            }
        );
        for text in [
            "Hello world",
            "This text mentions wants you to sign in with your Solana account.",
            "Quoted message:\nexample.com wants you to sign in with your Solana account:",
        ] {
            assert_eq!(SiwsMessage::parse(text), Ok(None));
        }
        assert_eq!(
            SiwsMessage::parse("example.com wants you to sign in with your Solana account:"),
            Err("Invalid SIWS header".to_string())
        );
    }

    #[test]
    fn test_parse_optional_fields() {
        let minimal = SiwsMessage {
            domain: "example.com".to_string(),
            address: "AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9".to_string(),
            statement: None,
            uri: None,
            version: None,
            chain_id: None,
            nonce: None,
            issued_at: None,
            expiration_time: None,
            not_before: None,
            request_id: None,
            resources: vec![],
        };
        for (body, expected) in [
            ("", minimal.clone()),
            (
                "\n\nSign in.",
                SiwsMessage {
                    statement: Some("Sign in.".to_string()),
                    ..minimal.clone()
                },
            ),
            (
                "\n\nVersion: 1",
                SiwsMessage {
                    version: Some("1".to_string()),
                    ..minimal.clone()
                },
            ),
            (
                "\n\nRequest ID: ",
                SiwsMessage {
                    request_id: Some(String::new()),
                    ..minimal.clone()
                },
            ),
            ("\n\nResources:", minimal),
        ] {
            assert_eq!(SiwsMessage::parse(&mock_siws_message(body)), Ok(Some(expected)));
        }
    }

    #[test]
    fn test_parse_invalid_fields() {
        for (body, error) in [
            ("\n\nVersion: 1\nVersion: 2", "Invalid SIWS field order or trailing content"),
            ("\n\nVersion: 1\nURI: https://example.com", "Invalid SIWS field order or trailing content"),
            ("\n\nSign in.\nNonce: hidden", "Invalid SIWS statement"),
            ("\nURI: https://example.com", "Invalid SIWS separator"),
            ("\n\nNonce:missing-space", "Invalid SIWS field order or trailing content"),
            ("\n\nVersion: 1\n", "Invalid SIWS field order or trailing content"),
            ("\n\nResources:\nNonce: hidden", "Invalid SIWS resource"),
            ("\n\n", "Invalid SIWS statement"),
            ("\n\nSign\tin.", "Invalid SIWS statement"),
            ("\n\nSign in.\u{7f}", "Invalid SIWS statement"),
            ("\n\nSign in café.", "Invalid SIWS statement"),
        ] {
            assert_eq!(SiwsMessage::parse(&mock_siws_message(body)), Err(error.to_string()));
        }
    }
}
