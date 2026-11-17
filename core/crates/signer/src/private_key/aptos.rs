use primitives::decode_hex;
use primitives::hex::encode_with_0x;
use zeroize::Zeroizing;

use crate::SignatureScheme;

const PRIVATE_KEY_INFIX: &str = "-priv-";

pub(super) fn decode_prefixed_hex(value: &str, scheme: SignatureScheme) -> Option<Vec<u8>> {
    decode_hex(value.strip_prefix(scheme.as_ref())?.strip_prefix(PRIVATE_KEY_INFIX)?).ok()
}

pub(super) fn encode_prefixed_hex(bytes: &[u8], scheme: SignatureScheme) -> String {
    let key = Zeroizing::new(encode_with_0x(bytes));
    format!("{}{}{}", scheme.as_ref(), PRIVATE_KEY_INFIX, key.as_str())
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, hex};

    use crate::{decode_private_key, encode_private_key};

    #[test]
    fn test_decode_aptos_prefixed_hex() {
        let value = "ed25519-priv-0x0000000000000000000000000000000000000000000000000000000000000001";
        let bytes = decode_private_key(&Chain::Aptos, value).unwrap();

        assert_eq!(hex::encode(bytes.as_slice()), "0000000000000000000000000000000000000000000000000000000000000001");
        assert_eq!(encode_private_key(&Chain::Aptos, &bytes).unwrap(), value);
        assert_eq!(decode_private_key(&Chain::Aptos, "ed25519-priv-0000000000000000000000000000000000000000000000000000000000000001").unwrap(), bytes);

        assert_eq!(decode_private_key(&Chain::Aptos, "0x0000000000000000000000000000000000000000000000000000000000000001").unwrap(), bytes);
        assert!(decode_private_key(&Chain::Aptos, "secp256k1-priv-0x0000000000000000000000000000000000000000000000000000000000000001").is_err());
        assert!(decode_private_key(&Chain::Aptos, "ed25519-pub-0x0000000000000000000000000000000000000000000000000000000000000001").is_err());
        assert!(decode_private_key(&Chain::Aptos, "ed25519-priv-0x00000000000000000000000000000001").is_err());
        assert!(decode_private_key(&Chain::Aptos, "ed25519-priv-").is_err());
        assert!(decode_private_key(&Chain::Sui, value).is_err());
        assert!(decode_private_key(&Chain::Ethereum, value).is_err());
    }
}
