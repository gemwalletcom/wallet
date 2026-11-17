use gem_encoding::{decode_strkey, encode_strkey};
use zeroize::Zeroizing;

use crate::SignerError;

const SECRET_SEED_VERSION: u8 = 0x90;

pub(super) fn decode(value: &str) -> Option<Zeroizing<Vec<u8>>> {
    decode_strkey(value, SECRET_SEED_VERSION).ok()
}

pub(super) fn encode(private_key: &[u8]) -> Result<String, SignerError> {
    let seed = private_key.try_into().map_err(|_| SignerError::invalid_input("Invalid Stellar private key"))?;
    Ok(encode_strkey(SECRET_SEED_VERSION, seed))
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, hex};

    use crate::{decode_private_key, encode_private_key};

    #[test]
    fn test_stellar_private_key() {
        let value = "SA6XNHUKMW4QAKSHB2NOZ4SYP34ERYVAWSBTEDREYSJ2LEJ5LFHLTIRJ";
        let key = "3d769e8a65b9002a470e9aecf2587ef848e2a0b483320e24c493a5913d594eb9";
        let private_key = decode_private_key(&Chain::Stellar, value).unwrap();

        assert_eq!(hex::encode(private_key.as_slice()), key);
        assert_eq!(encode_private_key(&Chain::Stellar, &private_key).unwrap(), value);
        assert_eq!(decode_private_key(&Chain::Stellar, key).unwrap(), private_key);

        assert!(encode_private_key(&Chain::Stellar, &[1u8; 16]).is_err());
        assert!(decode_private_key(&Chain::Stellar, "SA6XNHUKMW4QAKSHB2NOZ4SYP34ERYVAWSBTEDREYSJ2LEJ5LFHLTIRA").is_err());
        assert!(decode_private_key(&Chain::Stellar, "GA6XNHUKMW4QAKSHB2NOZ4SYP34ERYVAWSBTEDREYSJ2LEJ5LFHLTIRJ").is_err());
    }
}
