use primitives::decode_hex;
use primitives::hex::encode_with_0x;
use zeroize::Zeroizing;

const ED25519_PRIVATE_KEY_PREFIX: &str = "ed25519-priv-";

pub(super) fn decode(value: &str) -> Option<Zeroizing<Vec<u8>>> {
    decode_hex(value.strip_prefix(ED25519_PRIVATE_KEY_PREFIX)?).ok().map(Zeroizing::new)
}

pub(super) fn encode(private_key: &[u8]) -> String {
    let key = Zeroizing::new(encode_with_0x(private_key));
    format!("{ED25519_PRIVATE_KEY_PREFIX}{}", key.as_str())
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, hex};

    use crate::{decode_private_key, encode_private_key};

    #[test]
    fn test_aptos_private_key() {
        let key = "0000000000000000000000000000000000000000000000000000000000000001";
        let value = format!("ed25519-priv-0x{key}");
        let private_key = decode_private_key(&Chain::Aptos, &value).unwrap();

        assert_eq!(hex::encode(private_key.as_slice()), key);
        assert_eq!(encode_private_key(&Chain::Aptos, &private_key).unwrap(), value);
        assert_eq!(decode_private_key(&Chain::Aptos, &format!("ed25519-priv-{key}")).unwrap(), private_key);
        assert_eq!(decode_private_key(&Chain::Aptos, &format!("0x{key}")).unwrap(), private_key);

        assert!(decode_private_key(&Chain::Aptos, &format!("secp256k1-priv-0x{key}")).is_err());
        assert!(decode_private_key(&Chain::Aptos, &format!("ed25519-pub-0x{key}")).is_err());
        assert!(decode_private_key(&Chain::Aptos, &format!("ed25519-priv-0x{}", &key[..32])).is_err());
        assert!(decode_private_key(&Chain::Aptos, "ed25519-priv-").is_err());
        assert!(decode_private_key(&Chain::Sui, &value).is_err());
        assert!(decode_private_key(&Chain::Ethereum, &value).is_err());
    }
}
