use chrono::Utc;
use gem_solana::siws::SiwsMessage;
use url::Url;

use crate::{SignDigestType, SignMessageValidation};

pub fn validate_sign_message_account(sign_type: &SignDigestType, data: &str, address: &str) -> Result<(), String> {
    if *sign_type == SignDigestType::Base58
        && let Some(message) = SiwsMessage::decode_base58(data)?
        && message.address != address
    {
        return Err("SIWS address does not match signing account".to_string());
    }
    Ok(())
}

pub(crate) fn validate(input: &SignMessageValidation) -> Result<(), String> {
    let Some(message) = SiwsMessage::decode_base58(input.data)? else {
        return Ok(());
    };
    message.validate(input.chain, Utc::now())?;
    validate_origin(&message, input.session_domain)
}

fn validate_origin(message: &SiwsMessage, session_domain: &str) -> Result<(), String> {
    let session = Url::parse(session_domain).map_err(|_| "Invalid session origin")?;
    if !matches!(session.scheme(), "https" | "http") || session.host_str().is_none() {
        return Err("Invalid session origin".to_string());
    }
    let domain = Url::parse(&format!("{}://{}", session.scheme(), message.domain)).map_err(|_| "Invalid SIWS domain")?;
    if domain.origin() != session.origin() {
        return Err("SIWS domain does not match session origin".to_string());
    }
    if let Some(uri) = &message.uri
        && Url::parse(uri).map_err(|_| "Invalid SIWS URI")?.origin() != session.origin()
    {
        return Err("SIWS URI does not match session origin".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS};

    use super::*;
    use crate::validate_sign_message;

    #[test]
    fn test_validate_sign_message() {
        let data = bs58::encode(include_str!("../../gem_solana/testdata/siws_sign_in.txt")).into_string();
        assert_eq!(
            validate_sign_message(&SignMessageValidation {
                chain: Chain::Solana,
                sign_type: &SignDigestType::Base58,
                data: &data,
                session_domain: "https://example.com",
            }),
            Ok(())
        );
        assert_eq!(validate_sign_message_account(&SignDigestType::Base58, &data, TEST_PRIVATE_KEY_SOLANA_ADDRESS), Ok(()));
        assert_eq!(
            validate_sign_message_account(&SignDigestType::Base58, &data, "other"),
            Err("SIWS address does not match signing account".to_string())
        );
    }

    #[test]
    fn test_validate_origin() {
        let message = SiwsMessage::parse(include_str!("../../gem_solana/testdata/siws_sign_in.txt")).unwrap().unwrap();
        let domain = message.domain.as_str();
        let uri = message.uri.as_deref().unwrap();
        let session = format!("https://{domain}");
        let domain_with_port = format!("{domain}:8443");
        let session_with_port = format!("https://{domain_with_port}");
        let uri_with_port = format!("{session_with_port}/login");
        let http_uri = format!("http://{domain}/login");
        for (domain, uri, session, expected) in [
            (domain, uri, session.as_str(), Ok(())),
            (domain, uri, "https://other.xyz", Err("SIWS domain does not match session origin")),
            (domain_with_port.as_str(), uri, session.as_str(), Err("SIWS domain does not match session origin")),
            (domain, "https://other.xyz/login", session.as_str(), Err("SIWS URI does not match session origin")),
            (domain, http_uri.as_str(), session.as_str(), Err("SIWS URI does not match session origin")),
            (domain_with_port.as_str(), uri_with_port.as_str(), session_with_port.as_str(), Ok(())),
        ] {
            let message = SiwsMessage {
                domain: domain.to_string(),
                uri: Some(uri.to_string()),
                ..message.clone()
            };
            assert_eq!(validate_origin(&message, session), expected.map_err(str::to_string));
        }
    }

    #[test]
    fn test_validate_sign_message_account_ignores_non_siws() {
        for bytes in [b"ordinary message".as_slice(), &[0xff, 0x00]] {
            let data = bs58::encode(bytes).into_string();
            assert_eq!(validate_sign_message_account(&SignDigestType::Base58, &data, "other"), Ok(()));
        }
    }
}
