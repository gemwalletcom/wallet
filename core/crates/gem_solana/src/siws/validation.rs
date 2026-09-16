use chrono::{DateTime, Utc};
use gem_encoding::is_valid_percent_encoding;
use primitives::{Chain, domain::parse_domain};
use url::Url;

use super::SiwsMessage;
use crate::validate_address;

const MIN_NONCE_LENGTH: usize = 8;
const REQUEST_ID_CHARACTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~!$&'()*+,;=:@%";

impl SiwsMessage {
    pub fn validate(&self, chain: Chain, now: DateTime<Utc>) -> Result<(), String> {
        if chain != Chain::Solana {
            return Err("Unsupported chain for SIWS".to_string());
        }

        if !validate_address(&self.address) {
            return Err("Invalid SIWS address".to_string());
        }

        parse_domain(&self.domain).ok_or("Invalid SIWS domain")?;

        if self.version.as_ref().is_some_and(|value| value != "1") {
            return Err("Unsupported SIWS version".to_string());
        }

        if let Some(chain_id) = &self.chain_id {
            let reference = chain_id.strip_prefix("solana:").unwrap_or(chain_id);
            if reference != "mainnet" && reference != "mainnet-beta" && reference != chain.network_id() && Some(reference) != chain.network_id().get(..32) {
                return Err("Chain ID mismatch".to_string());
            }
        }

        if self
            .nonce
            .as_ref()
            .is_some_and(|value| value.len() < MIN_NONCE_LENGTH || !value.bytes().all(|byte| byte.is_ascii_alphanumeric()))
        {
            return Err("Invalid SIWS nonce".to_string());
        }

        for uri in self.uri.iter().chain(self.resources.iter()) {
            if !uri.is_ascii() || uri.chars().any(|character| character.is_whitespace() || character.is_control()) || !is_valid_percent_encoding(uri) || Url::parse(uri).is_err() {
                return Err("Invalid SIWS URI".to_string());
            }
        }

        self.validate_request_id()?;

        let [issued_at, expiration_time, not_before] = [&self.issued_at, &self.expiration_time, &self.not_before]
            .map(|value| value.as_deref().map(DateTime::parse_from_rfc3339).transpose().map_err(|_| "Invalid SIWS timestamp"));
        let (issued_at, expiration_time, not_before) = (issued_at?, expiration_time?, not_before?);

        if expiration_time.is_some_and(|expiration| expiration <= now || issued_at.is_some_and(|issued| issued >= expiration)) {
            return Err("SIWS message expired or invalid expiration".to_string());
        }

        if not_before.is_some_and(|start| start > now) {
            return Err("SIWS message not yet valid".to_string());
        }

        Ok(())
    }

    fn validate_request_id(&self) -> Result<(), String> {
        let Some(request_id) = &self.request_id else {
            return Ok(());
        };
        if !is_valid_percent_encoding(request_id) || !request_id.bytes().all(|byte| REQUEST_ID_CHARACTERS.contains(&byte)) {
            return Err("Invalid SIWS request ID".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::SiwsMessage;

    #[test]
    fn test_validate() {
        let now = "2026-09-01T12:00:00Z".parse().unwrap();
        assert_eq!(SiwsMessage::mock_complete().validate(Chain::Solana, now), Ok(()));
        assert_eq!(SiwsMessage::mock_complete().validate(Chain::Ethereum, now), Err("Unsupported chain for SIWS".to_string()));
        for (invalid, error) in [
            (
                SiwsMessage {
                    address: "111".to_string(),
                    ..SiwsMessage::mock_complete()
                },
                "Invalid SIWS address",
            ),
            (
                SiwsMessage {
                    domain: "example.com/path".to_string(),
                    ..SiwsMessage::mock_complete()
                },
                "Invalid SIWS domain",
            ),
            (
                SiwsMessage {
                    domain: "user@example.com".to_string(),
                    ..SiwsMessage::mock_complete()
                },
                "Invalid SIWS domain",
            ),
            (
                SiwsMessage {
                    version: Some("2".to_string()),
                    ..SiwsMessage::mock_complete()
                },
                "Unsupported SIWS version",
            ),
            (
                SiwsMessage {
                    chain_id: Some("devnet".to_string()),
                    ..SiwsMessage::mock_complete()
                },
                "Chain ID mismatch",
            ),
            (
                SiwsMessage {
                    nonce: Some("short".to_string()),
                    ..SiwsMessage::mock_complete()
                },
                "Invalid SIWS nonce",
            ),
            (
                SiwsMessage {
                    nonce: Some("invalid!".to_string()),
                    ..SiwsMessage::mock_complete()
                },
                "Invalid SIWS nonce",
            ),
            (
                SiwsMessage {
                    uri: Some("relative/path".to_string()),
                    ..SiwsMessage::mock_complete()
                },
                "Invalid SIWS URI",
            ),
            (
                SiwsMessage {
                    resources: vec!["https://example.com/%ZZ".to_string()],
                    ..SiwsMessage::mock_complete()
                },
                "Invalid SIWS URI",
            ),
            (
                SiwsMessage {
                    request_id: Some("invalid%".to_string()),
                    ..SiwsMessage::mock_complete()
                },
                "Invalid SIWS request ID",
            ),
            (
                SiwsMessage {
                    issued_at: Some("invalid".to_string()),
                    ..SiwsMessage::mock_complete()
                },
                "Invalid SIWS timestamp",
            ),
        ] {
            assert_eq!(invalid.validate(Chain::Solana, now), Err(error.to_string()));
        }
        for chain_id in ["mainnet", "solana:mainnet", "mainnet-beta", "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp", Chain::Solana.network_id()] {
            assert_eq!(
                SiwsMessage {
                    chain_id: Some(chain_id.to_string()),
                    ..SiwsMessage::mock_complete()
                }
                .validate(Chain::Solana, now),
                Ok(())
            );
        }
    }

    #[test]
    fn test_validate_timestamps() {
        assert_eq!(
            SiwsMessage {
                issued_at: Some("2026-09-01T21:00:00+09:00".to_string()),
                expiration_time: Some("2026-09-02T21:00:00+09:00".to_string()),
                not_before: Some("2026-09-01T20:00:00+09:00".to_string()),
                ..SiwsMessage::mock_complete()
            }
            .validate(Chain::Solana, "2026-09-01T12:00:00Z".parse().unwrap()),
            Ok(())
        );

        for (now, expected) in [
            ("2026-09-01T10:59:59Z", Err("SIWS message not yet valid".to_string())),
            ("2026-09-01T11:00:00Z", Ok(())),
            ("2026-09-02T11:59:59.999Z", Ok(())),
            ("2026-09-02T12:00:00Z", Err("SIWS message expired or invalid expiration".to_string())),
        ] {
            assert_eq!(SiwsMessage::mock_complete().validate(Chain::Solana, now.parse().unwrap()), expected);
        }
        assert_eq!(
            SiwsMessage {
                expiration_time: Some("2026-09-01T12:00:00Z".to_string()),
                ..SiwsMessage::mock_complete()
            }
            .validate(Chain::Solana, "2026-09-01T11:00:00Z".parse().unwrap()),
            Err("SIWS message expired or invalid expiration".to_string())
        );
    }
}
