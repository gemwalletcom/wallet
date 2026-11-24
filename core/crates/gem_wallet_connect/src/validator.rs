use primitives::unix_timestamp;

use crate::actions::WalletConnectTransactionType;
use crate::decode::decode_message_bytes;
use crate::sign_type::SignDigestType;
use crate::siws;
use gem_evm::siwe::SiweMessage;
use primitives::Chain;
use sign_in::{host_only, parse_domain};

pub struct SignMessageValidation<'a> {
    pub chain: Chain,
    pub sign_type: &'a SignDigestType,
    pub data: &'a str,
    pub session_domain: &'a str,
}

fn current_timestamp() -> i64 {
    unix_timestamp() as i64
}

pub fn validate_sign_message(input: &SignMessageValidation) -> Result<(), String> {
    match input.sign_type {
        SignDigestType::Eip712 => {
            let expected_chain_id = input.chain.network_id().parse::<u64>().map_err(|_| format!("Chain {} does not have a numeric network ID", input.chain))?;
            gem_evm::eip712::validate_eip712_chain_id(input.data, expected_chain_id)
        }
        SignDigestType::TonPersonal => {
            gem_ton::signer::TonSignMessageData::from_bytes(input.data.as_bytes()).map_err(|e| e.to_string())?;
            Ok(())
        }
        SignDigestType::Eip191 | SignDigestType::Siwe => validate_siwe(input),
        SignDigestType::Base58 => siws::validate(input),
        SignDigestType::SuiPersonal | SignDigestType::TronPersonal => Ok(()),
    }
}

fn validate_siwe(input: &SignMessageValidation) -> Result<(), String> {
    let text = String::from_utf8(decode_message_bytes(input.data)).ok();
    let Some(message) = text.as_deref().and_then(SiweMessage::try_parse) else {
        if *input.sign_type == SignDigestType::Siwe || text.as_deref().is_some_and(SiweMessage::is_sign_in) {
            return Err("Invalid SIWE message".to_string());
        }
        return Ok(());
    };
    message.validate(input.chain)?;
    validate_domain(&message.domain, input.session_domain)
}

pub fn validate_domain(domain: &str, session_domain: &str) -> Result<(), String> {
    let session_host = host_only(session_domain).ok_or_else(|| "Invalid session origin".to_string())?;
    let message_url = parse_domain(domain).ok_or_else(|| "Invalid SIWE domain".to_string())?;
    if message_url.host_str() != Some(session_host.as_str()) {
        return Err(format!("Domain mismatch: SIWE domain {domain} does not match session origin {session_domain}"));
    }
    Ok(())
}

pub fn validate_send_transaction(transaction_type: &WalletConnectTransactionType, data: &str) -> Result<(), String> {
    let WalletConnectTransactionType::Ton { .. } = transaction_type else {
        return Ok(());
    };

    let json: serde_json::Value = serde_json::from_str(data).map_err(|_| "Invalid JSON".to_string())?;

    if let Some(valid_until) = json.get("valid_until").and_then(serde_json::Value::as_i64)
        && current_timestamp() >= valid_until
    {
        return Err("Transaction expired".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_evm::testkit::eip712_mock::mock_eip712_json;
    use gem_evm::testkit::siwe_mock::{mock_siwe_message, mock_siwe_message_hex};
    use primitives::TransferDataOutputType;

    #[test]
    fn test_validate_eip712_chain_match() {
        assert!(validate_sign_message(&SignMessageValidation::mock(Chain::Ethereum, &SignDigestType::Eip712, &mock_eip712_json(1), "")).is_ok());
    }

    #[test]
    fn test_validate_eip712_chain_mismatch() {
        let result = validate_sign_message(&SignMessageValidation::mock(Chain::Ethereum, &SignDigestType::Eip712, &mock_eip712_json(137), ""));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain ID mismatch"));
    }

    #[test]
    fn test_validate_eip712_without_domain_chain_id() {
        let data = include_str!("../../gem_evm/testdata/ens_upload_avatar.json");
        assert!(validate_sign_message(&SignMessageValidation::mock(Chain::Ethereum, &SignDigestType::Eip712, data, "")).is_ok());
    }

    #[test]
    fn test_validate_eip712_domain_chain_id_without_schema_rejects() {
        let data = include_str!("../../gem_evm/testdata/eip712_domain_chain_id_without_schema_field.json");
        let result = validate_sign_message(&SignMessageValidation::mock(Chain::Ethereum, &SignDigestType::Eip712, data, ""));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("chainId"));
    }

    #[test]
    fn test_validate_eip191_always_ok() {
        assert!(validate_sign_message(&SignMessageValidation::mock(Chain::Ethereum, &SignDigestType::Eip191, "anything", "example.com")).is_ok());
    }

    #[test]
    fn test_validate_ton_send_transaction_expired() {
        let ton_type = WalletConnectTransactionType::Ton {
            output_type: TransferDataOutputType::EncodedTransaction,
        };
        assert!(validate_send_transaction(&ton_type, r#"{"valid_until": 1234567890, "messages": []}"#).is_err());
    }

    #[test]
    fn test_validate_ton_send_transaction_valid() {
        let ton_type = WalletConnectTransactionType::Ton {
            output_type: TransferDataOutputType::EncodedTransaction,
        };
        assert!(validate_send_transaction(&ton_type, r#"{"valid_until": 9999999999, "messages": []}"#).is_ok());
    }

    #[test]
    fn test_validate_ethereum_send_transaction_always_ok() {
        assert!(validate_send_transaction(&WalletConnectTransactionType::Ethereum, "{}").is_ok());
    }

    #[test]
    fn test_validate_ton_send_transaction_no_expiry() {
        let ton_type = WalletConnectTransactionType::Ton {
            output_type: TransferDataOutputType::EncodedTransaction,
        };
        assert!(validate_send_transaction(&ton_type, r#"{"messages": []}"#).is_ok());
    }

    #[test]
    fn test_validate_ton_sign_message() {
        use gem_ton::signer::{TonSignDataPayload, TonSignMessageData};

        assert!(validate_sign_message(&SignMessageValidation::mock(Chain::Ton, &SignDigestType::TonPersonal, r#"{"payload":{"text":"Hello"},"domain":"example.com"}"#, "")).is_err());

        assert!(validate_sign_message(&SignMessageValidation::mock(Chain::Ton, &SignDigestType::TonPersonal, r#"{"payload":{"type":"unknown"},"domain":"example.com"}"#, "")).is_err());

        let ton_data = TonSignMessageData::new(
            TonSignDataPayload::Text { text: "Hello".to_string() },
            "example.com".to_string(),
            "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg".to_string(),
        );
        assert!(validate_sign_message(&SignMessageValidation::mock(Chain::Ton, &SignDigestType::TonPersonal, &String::from_utf8(ton_data.to_bytes()).unwrap(), "")).is_ok());

        let ton_data = TonSignMessageData::new(
            TonSignDataPayload::Binary { bytes: "SGVsbG8=".to_string() },
            "example.com".to_string(),
            "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg".to_string(),
        );
        assert!(validate_sign_message(&SignMessageValidation::mock(Chain::Ton, &SignDigestType::TonPersonal, &String::from_utf8(ton_data.to_bytes()).unwrap(), "")).is_ok());

        let ton_data = TonSignMessageData::new(
            TonSignDataPayload::Cell {
                schema: "comment#00000000 text:SnakeData = InMsgBody;".to_string(),
                cell: "te6c".to_string(),
            },
            "example.com".to_string(),
            "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg".to_string(),
        );
        assert!(validate_sign_message(&SignMessageValidation::mock(Chain::Ton, &SignDigestType::TonPersonal, &String::from_utf8(ton_data.to_bytes()).unwrap(), "")).is_ok());
    }

    #[test]
    fn test_validate_siwe() {
        let valid = mock_siwe_message("thepoc.xyz", 1);
        assert!(validate_sign_message(&SignMessageValidation::mock(Chain::Ethereum, &SignDigestType::Siwe, &valid, "https://thepoc.xyz")).is_ok());

        let with_port = mock_siwe_message("thepoc.xyz:8080", 1);
        assert!(validate_sign_message(&SignMessageValidation::mock(Chain::Ethereum, &SignDigestType::Siwe, &with_port, "https://thepoc.xyz")).is_ok());

        let chain_mismatch = mock_siwe_message("thepoc.xyz", 137);
        assert!(
            validate_sign_message(&SignMessageValidation::mock(Chain::Ethereum, &SignDigestType::Siwe, &chain_mismatch, "https://thepoc.xyz"))
                .unwrap_err()
                .contains("Chain ID mismatch")
        );

        let domain_mismatch = mock_siwe_message("evil.com", 1);
        assert!(
            validate_sign_message(&SignMessageValidation::mock(Chain::Ethereum, &SignDigestType::Siwe, &domain_mismatch, "https://thepoc.xyz"))
                .unwrap_err()
                .contains("Domain mismatch")
        );

        assert!(
            validate_sign_message(&SignMessageValidation::mock(Chain::Ethereum, &SignDigestType::Siwe, "not siwe", "https://thepoc.xyz"))
                .unwrap_err()
                .contains("Invalid SIWE message")
        );
    }

    #[test]
    fn test_validate_domain() {
        for domain in ["example.com", "EXAMPLE.com", "example.com:8443"] {
            assert_eq!(validate_domain(domain, "https://example.com"), Ok(()), "{domain}");
        }
        assert!(validate_domain("app.uniswap.org", "https://example.com").unwrap_err().contains("Domain mismatch"));

        for domain in [
            "",
            "https://example.com",
            "example.com/login",
            "app.uniswap.org@example.com",
            "app.uniswap.org wants you to sign in with your Ethereum account:\n@example.com",
        ] {
            assert_eq!(
                validate_domain(domain, "https://example.com"),
                Err("Invalid SIWE domain".to_string()),
                "a sign-in message opens with the domain as written, so only a host with an optional port names a site: {domain}"
            );
        }
    }

    #[test]
    fn test_validate_eip191_siwe() {
        let other_site = mock_siwe_message("evil.com", 1);
        for (data, session_domain, error) in [
            (mock_siwe_message_hex("thepoc.xyz", 137), "https://thepoc.xyz", "Chain ID mismatch"),
            (mock_siwe_message_hex("evil.com", 1), "https://thepoc.xyz", "Domain mismatch"),
            (mock_siwe_message("0xother.xyz", 1), "https://evil.com", "Domain mismatch"),
            (other_site.replace("Chain ID: 1", "Chain ID: 99999999999999999999999"), "https://thepoc.xyz", "Chain ID mismatch"),
            (other_site.replace("URI: https://evil.com", "URI:  "), "https://thepoc.xyz", "Invalid SIWE message"),
            (other_site.replace("Issued At: 2026-03-06T01:56:42.927Z", "Issued At:  "), "https://thepoc.xyz", "Invalid SIWE message"),
            (other_site.replace("Nonce: gmdhs9w9yfrl2kf2\n", ""), "https://thepoc.xyz", "Invalid SIWE message"),
        ] {
            let result = validate_sign_message(&SignMessageValidation::mock(Chain::Ethereum, &SignDigestType::Eip191, &data, session_domain));
            assert!(result.unwrap_err().starts_with(error), "a message that opens as a sign-in is signed only for the app asking: {data}");
        }

        for (data, session_domain) in [(mock_siwe_message_hex("thepoc.xyz", 1), "https://thepoc.xyz"), ("0x48656c6c6f".to_string(), "https://example.com")] {
            assert_eq!(validate_sign_message(&SignMessageValidation::mock(Chain::Ethereum, &SignDigestType::Eip191, &data, session_domain)), Ok(()), "{data}");
        }
    }
}
