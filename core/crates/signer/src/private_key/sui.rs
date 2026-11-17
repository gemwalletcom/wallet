use bech32::primitives::decode::CheckedHrpstring;
use bech32::{Bech32, Hrp};
use zeroize::Zeroizing;

use crate::SignerError;

const PRIVATE_KEY_HRP: Hrp = Hrp::parse_unchecked("suiprivkey");
const ED25519_FLAG: u8 = 0x00;

pub(super) fn decode(value: &str) -> Option<Zeroizing<Vec<u8>>> {
    let checked = CheckedHrpstring::new::<Bech32>(value).ok()?;
    if checked.hrp() != PRIVATE_KEY_HRP {
        return None;
    }
    let payload = Zeroizing::new(checked.byte_iter().collect::<Vec<u8>>());
    let (&ED25519_FLAG, private_key) = payload.split_first()? else {
        return None;
    };
    let canonical = Zeroizing::new(encode(private_key).ok()?);
    canonical.eq_ignore_ascii_case(value).then(|| Zeroizing::new(private_key.to_vec()))
}

pub(super) fn encode(private_key: &[u8]) -> Result<String, SignerError> {
    let mut payload = Zeroizing::new(Vec::with_capacity(private_key.len() + 1));
    payload.push(ED25519_FLAG);
    payload.extend_from_slice(private_key);
    bech32::encode::<Bech32>(PRIVATE_KEY_HRP, &payload).map_err(|_| SignerError::invalid_input("Invalid Sui private key"))
}

#[cfg(test)]
mod tests {
    use bech32::Bech32m;
    use primitives::{Chain, hex};

    use super::*;
    use crate::{decode_private_key, encode_private_key};

    #[test]
    fn test_sui_private_key() {
        let vectors = [
            ("suiprivkey1qzwant3kaegmjy4qxex93s0jzvemekkjmyv3r2sjwgnv2y479pgsywhveae", "9dd9ae36ee51b912a0364c58c1f21333bcdad2d91911aa127226c512be285102"),
            ("suiprivkey1qrh2sjl88rze74hwjndw3l26dqyz63tea5u9frtwcsqhmfk9vxdlx8cpv0g", "eea84be738c59f56ee94dae8fd5a68082d4579ed38548d6ec4017da6c5619bf3"),
            ("suiprivkey1qzg73qyvfz0wpnyectkl08nrhe4pgnu0vqx8gydu96qx7uj4wyr8gcrjlh3", "91e8808c489ee0cc99c2edf79e63be6a144f8f600c7411bc2e806f7255710674"),
        ];
        for (value, key) in vectors {
            let private_key = decode_private_key(&Chain::Sui, value).unwrap();

            assert_eq!(hex::encode(private_key.as_slice()), key);
            assert_eq!(encode_private_key(&Chain::Sui, &private_key).unwrap(), value);
            assert_eq!(decode_private_key(&Chain::Sui, &format!("0x{key}")).unwrap(), private_key);
            assert!(decode_private_key(&Chain::Aptos, value).is_err());
            assert!(decode_private_key(&Chain::Ethereum, value).is_err());
        }

        let ed25519_key = [[ED25519_FLAG].as_slice(), &[7u8; 32]].concat();
        let secp256k1_key = [[1u8].as_slice(), &[7u8; 32]].concat();
        let invalid = [
            bech32::encode::<Bech32>(PRIVATE_KEY_HRP, &secp256k1_key).unwrap(),
            bech32::encode::<Bech32>(PRIVATE_KEY_HRP, &ed25519_key[..32]).unwrap(),
            bech32::encode::<Bech32m>(PRIVATE_KEY_HRP, &ed25519_key).unwrap(),
            bech32::encode::<Bech32>(Hrp::parse_unchecked("sui"), &ed25519_key).unwrap(),
            "suiprivkey1qzwant3kaegmjy4qxex93s0jzvemekkjmyv3r2sjwgnv2y479pgsywhveaf".to_string(),
            "suiprivkey1qzwant3kaegmjy4qxex93s0jzvemekkjmyv3r2sjwgnv2y479pgs9npcvqt".to_string(),
        ];
        for value in invalid {
            assert!(decode_private_key(&Chain::Sui, &value).is_err());
        }
        assert!(decode_private_key(&Chain::Sui, &bech32::encode::<Bech32>(PRIVATE_KEY_HRP, &ed25519_key).unwrap()).is_ok());
    }
}
