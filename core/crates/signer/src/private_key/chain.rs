use primitives::{Chain, ChainType};
use zeroize::Zeroizing;

use super::encoding::KeyEncoding;
use crate::{CARDANO_EXTENDED_PRIVATE_KEY_LENGTH, Ed25519KeyPair, SignatureScheme, SignerError};

#[derive(Debug, Clone, Copy)]
enum KeyLayout {
    PrivateKey,
    Ed25519KeyPair,
}

fn import_encodings_for_chain(chain: &Chain) -> &'static [KeyEncoding] {
    match chain.chain_type() {
        ChainType::Bitcoin | ChainType::Cardano => &[],
        ChainType::Solana => &[KeyEncoding::Base58, KeyEncoding::Hex],
        ChainType::Stellar => &[KeyEncoding::StellarStrKey, KeyEncoding::Hex],
        ChainType::Near => &[KeyEncoding::PrefixedBase58(SignatureScheme::Ed25519)],
        ChainType::Aptos => &[KeyEncoding::PrefixedHex(SignatureScheme::Ed25519), KeyEncoding::Hex],
        ChainType::Sui => &[KeyEncoding::SuiBech32, KeyEncoding::Hex],
        _ => &[KeyEncoding::Hex],
    }
}

fn export_format_for_chain(chain: &Chain) -> (KeyLayout, KeyEncoding) {
    match chain.chain_type() {
        ChainType::Bitcoin => (KeyLayout::PrivateKey, KeyEncoding::Base58),
        ChainType::Solana => (KeyLayout::Ed25519KeyPair, KeyEncoding::Base58),
        ChainType::Near => (KeyLayout::Ed25519KeyPair, KeyEncoding::PrefixedBase58(SignatureScheme::Ed25519)),
        ChainType::Aptos => (KeyLayout::PrivateKey, KeyEncoding::PrefixedHex(SignatureScheme::Ed25519)),
        ChainType::Stellar => (KeyLayout::PrivateKey, KeyEncoding::StellarStrKey),
        ChainType::Sui => (KeyLayout::PrivateKey, KeyEncoding::SuiBech32),
        _ => (KeyLayout::PrivateKey, KeyEncoding::Hex),
    }
}

pub fn supports_private_key_import(chain: &Chain) -> bool {
    !import_encodings_for_chain(chain).is_empty()
}

fn scheme_for_chain(chain: &Chain) -> SignatureScheme {
    match chain.chain_type() {
        ChainType::Solana | ChainType::Ton | ChainType::Aptos | ChainType::Sui | ChainType::Near | ChainType::Stellar | ChainType::Algorand | ChainType::Polkadot => SignatureScheme::Ed25519,
        ChainType::Cardano => SignatureScheme::Ed25519CardanoExtended,
        _ => SignatureScheme::Secp256k1,
    }
}

fn validate_key(bytes: &[u8], scheme: SignatureScheme) -> Result<(), SignerError> {
    match scheme {
        SignatureScheme::Ed25519 => {
            let arr: &[u8; 32] = bytes.try_into().map_err(|_| SignerError::invalid_input("Invalid ed25519 private key"))?;
            ed25519_dalek::SigningKey::from_bytes(arr);
            Ok(())
        }
        SignatureScheme::Secp256k1 => crate::secp256k1::validate_private_key(bytes),
        SignatureScheme::Ed25519CardanoExtended => {
            if bytes.len() != CARDANO_EXTENDED_PRIVATE_KEY_LENGTH {
                return Err(SignerError::invalid_input("Invalid Cardano extended private key"));
            }
            Ok(())
        }
    }
}

pub fn decode_private_key(chain: &Chain, value: &str) -> Result<Zeroizing<Vec<u8>>, SignerError> {
    let scheme = scheme_for_chain(chain);
    import_encodings_for_chain(chain)
        .iter()
        .filter_map(|encoding| encoding.decode(value))
        .find(|private_key| validate_key(private_key, scheme).is_ok())
        .ok_or_else(|| SignerError::invalid_input("Invalid private key format"))
}

pub fn encode_private_key(chain: &Chain, private_key: &[u8]) -> Result<String, SignerError> {
    let (layout, encoding) = export_format_for_chain(chain);
    match layout {
        KeyLayout::PrivateKey => encoding.encode(private_key),
        KeyLayout::Ed25519KeyPair => {
            let keypair = Ed25519KeyPair::from_private_key(private_key)?;
            let mut bytes = Zeroizing::new(Vec::with_capacity(private_key.len() + keypair.public_key_bytes.len()));
            bytes.extend_from_slice(private_key);
            bytes.extend_from_slice(&keypair.public_key_bytes);
            encoding.encode(&bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use gem_encoding::{decode_base58, encode_base58};
    use primitives::hex;

    use super::*;

    #[test]
    fn test_decode_solana_base58() {
        let bytes = decode_private_key(&Chain::Solana, "4ha2npeRkDXipjgGJ3L5LhZ9TK9dRjP2yktydkFBhAzXj3N8ytpYyTS24kxcYGEefy4WKWRcog2zSPvpPZoGmxCC").unwrap();
        assert_eq!(bytes.len(), 32);
        assert_eq!(encode_private_key(&Chain::Solana, &bytes).unwrap(), "4ha2npeRkDXipjgGJ3L5LhZ9TK9dRjP2yktydkFBhAzXj3N8ytpYyTS24kxcYGEefy4WKWRcog2zSPvpPZoGmxCC");
    }

    #[test]
    fn test_decode_ethereum_hex() {
        let value = "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e";
        let bytes = decode_private_key(&Chain::Ethereum, value).unwrap();
        assert_eq!(bytes.len(), 32);
        assert_eq!(encode_private_key(&Chain::Ethereum, &bytes).unwrap(), value);
    }

    #[test]
    fn test_decode_near_prefixed_base58() {
        let private_key = [7u8; 32];
        let keypair = ed25519_dalek::SigningKey::from_bytes(&private_key).to_keypair_bytes();
        let value = format!("ed25519:{}", encode_base58(keypair));

        assert_eq!(encode_private_key(&Chain::Near, &private_key).unwrap(), value);
        assert_eq!(decode_private_key(&Chain::Near, &value).unwrap().as_slice(), private_key);
        assert!(decode_private_key(&Chain::Solana, &value).is_err());
        assert!(decode_private_key(&Chain::Ethereum, &value).is_err());
    }

    #[test]
    fn test_decode_rejects_addresses() {
        let public_key = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]).verifying_key().to_bytes();

        assert!(decode_private_key(&Chain::Solana, &encode_base58(public_key)).is_err());
        assert!(decode_private_key(&Chain::Near, &format!("ed25519:{}", encode_base58(public_key))).is_err());
        assert!(decode_private_key(&Chain::Near, &hex::encode(public_key)).is_err());
    }

    #[test]
    fn test_decode_invalid() {
        let mut mismatched_keypair = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]).to_keypair_bytes();
        mismatched_keypair[63] ^= 1;

        assert!(decode_private_key(&Chain::Ethereum, "not_valid").is_err());
        assert!(decode_private_key(&Chain::Stellar, "GA6XNHUKMW4QAKSHB2NOZ4SYP34ERYVAWSBTEDREYSJ2LEJ5LFHLTIRJ").is_err());
        assert!(decode_private_key(&Chain::Near, "ed25519:").is_err());
        assert!(decode_private_key(&Chain::Near, "secp256k1:11111111111111111111111111111111").is_err());
        assert!(decode_private_key(&Chain::Near, "ed25519:1111111111111111111111111111111").is_err());
        assert!(decode_private_key(&Chain::Near, &format!("ed25519:{}", encode_base58(mismatched_keypair))).is_err());
    }

    #[test]
    fn test_encode_solana_base58() {
        let bytes = decode_private_key(&Chain::Solana, "4ha2npeRkDXipjgGJ3L5LhZ9TK9dRjP2yktydkFBhAzXj3N8ytpYyTS24kxcYGEefy4WKWRcog2zSPvpPZoGmxCC").unwrap();
        let encoded = encode_private_key(&Chain::Solana, &bytes).unwrap();

        assert_eq!(encoded, "4ha2npeRkDXipjgGJ3L5LhZ9TK9dRjP2yktydkFBhAzXj3N8ytpYyTS24kxcYGEefy4WKWRcog2zSPvpPZoGmxCC");
    }

    #[test]
    fn test_encode_solana_seed_round_trip() {
        let seed = [7u8; 32];
        let exported = encode_private_key(&Chain::Solana, &seed).unwrap();
        let keypair = decode_base58(&exported).unwrap();

        assert_eq!(keypair.len(), 64);
        assert_eq!(&keypair[..32], seed);
        assert_eq!(&keypair[32..], ed25519_dalek::SigningKey::from_bytes(&seed).verifying_key().as_bytes());
        assert_eq!(decode_private_key(&Chain::Solana, &exported).unwrap().as_slice(), seed);
    }

    #[test]
    fn test_encode_ethereum_hex() {
        let bytes = decode_private_key(&Chain::Ethereum, "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e").unwrap();
        let encoded = encode_private_key(&Chain::Ethereum, &bytes).unwrap();

        assert_eq!(encoded, "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e");
    }

    #[test]
    fn test_encode_bitcoin_base58() {
        let bytes = decode_private_key(&Chain::Ethereum, "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e").unwrap();
        let encoded = encode_private_key(&Chain::Bitcoin, &bytes).unwrap();

        assert_eq!(encoded, "4Hmr8TxnwVB7m6fzfPRcASMn2hLRgzUz3gDGwF4ZnpVK");
    }

    #[test]
    fn test_encode_hex_preserves_bytes() {
        assert_eq!(encode_private_key(&Chain::Ethereum, &[1u8; 16]).unwrap(), "0x01010101010101010101010101010101");
    }

    #[test]
    fn test_supports_private_key_import() {
        assert!(supports_private_key_import(&Chain::Ethereum));
        assert!(supports_private_key_import(&Chain::Solana));
        assert!(supports_private_key_import(&Chain::Stellar));
        assert!(!supports_private_key_import(&Chain::Cardano));
        assert!(!supports_private_key_import(&Chain::Bitcoin));
        assert!(!supports_private_key_import(&Chain::Litecoin));
        assert!(!supports_private_key_import(&Chain::Doge));
    }
}
