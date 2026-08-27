use alloy_primitives::{hex, keccak256};
use primitives::{AuthMessage, ChainType};
use signer::Signer;

pub struct AuthMessageData {
    pub message: String,
    pub hash: [u8; 32],
}

pub fn create_auth_hash(auth_message: &AuthMessage) -> AuthMessageData {
    let message = serde_json::to_string(auth_message).unwrap_or_default();
    let hash = keccak256(message.as_bytes());
    AuthMessageData { message, hash: hash.into() }
}

pub fn verify_auth_signature(auth_message: &AuthMessage, signature: &str) -> bool {
    match auth_message.chain.chain_type() {
        ChainType::Ethereum => verify_ethereum_signature(auth_message, signature),
        _ => false, // TODO: Add support for other chain types
    }
}

fn verify_ethereum_signature(auth_message: &AuthMessage, signature: &str) -> bool {
    let data = create_auth_hash(auth_message);
    verify_hash_signature(&data.hash, signature, &auth_message.address)
}

fn verify_hash_signature(hash: &[u8; 32], signature: &str, expected_address: &str) -> bool {
    let Some(recovered) = recover_address_from_hash(hash, signature) else {
        return false;
    };
    recovered == expected_address
}

fn recover_address_from_hash(hash: &[u8; 32], signature: &str) -> Option<String> {
    let signature_bytes = hex::decode(signature.strip_prefix("0x").unwrap_or(signature)).ok()?;

    if signature_bytes.len() != 65 {
        return None;
    }

    Signer::recover_ethereum_address(hash, &signature_bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{
        AuthNonce, Chain,
        hex::encode_with_0x,
        testkit::signer_mock::{TEST_PRIVATE_KEY, TEST_PRIVATE_KEY_ETHEREUM_ADDRESS},
    };
    use signer::SignatureScheme;

    fn sign_auth_message(auth_message: &AuthMessage) -> String {
        let hash = create_auth_hash(auth_message).hash;
        let signature = Signer::sign_digest(SignatureScheme::Secp256k1, &hash, &TEST_PRIVATE_KEY).unwrap();
        encode_with_0x(&signature)
    }

    #[test]
    fn test_verify_auth_signature_success() {
        let auth_message = AuthMessage {
            chain: Chain::Ethereum,
            address: TEST_PRIVATE_KEY_ETHEREUM_ADDRESS.to_string(),
            auth_nonce: AuthNonce {
                nonce: "test-nonce-123".to_string(),
                timestamp: 1734100000,
            },
        };

        let signature = sign_auth_message(&auth_message);
        assert!(verify_auth_signature(&auth_message, &signature));
    }

    #[test]
    fn test_verify_auth_signature_invalid() {
        let auth_message = AuthMessage {
            chain: Chain::Ethereum,
            address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            auth_nonce: AuthNonce {
                nonce: "test123".to_string(),
                timestamp: 1234567890,
            },
        };
        assert!(!verify_auth_signature(&auth_message, "0x"));
    }
}
