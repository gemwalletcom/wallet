use bech32::primitives::decode::CheckedHrpstring;
use bech32::{Bech32, Hrp};
use zeroize::Zeroizing;

use crate::SignerError;

const SUI_PRIVATE_KEY_HRP: Hrp = Hrp::parse_unchecked("suiprivkey");
const SUI_ED25519_FLAG: u8 = 0x00;

pub(super) fn decode_sui_bech32(value: &str) -> Option<Vec<u8>> {
    let checked = CheckedHrpstring::new::<Bech32>(value).ok()?;
    if checked.hrp() != SUI_PRIVATE_KEY_HRP {
        return None;
    }
    let payload = Zeroizing::new(checked.byte_iter().collect::<Vec<u8>>());
    let (&SUI_ED25519_FLAG, private_key) = payload.split_first()? else {
        return None;
    };
    let canonical = Zeroizing::new(encode_sui_bech32(private_key).ok()?);
    canonical.eq_ignore_ascii_case(value).then(|| private_key.to_vec())
}

pub(super) fn encode_sui_bech32(private_key: &[u8]) -> Result<String, SignerError> {
    let mut payload = Zeroizing::new(Vec::with_capacity(private_key.len() + 1));
    payload.push(SUI_ED25519_FLAG);
    payload.extend_from_slice(private_key);
    bech32::encode::<Bech32>(SUI_PRIVATE_KEY_HRP, &payload).map_err(|_| SignerError::invalid_input("Invalid Sui private key"))
}

#[cfg(test)]
mod tests {
    use bech32::Bech32m;
    use primitives::{Chain, hex};

    use super::*;
    use crate::{decode_private_key, encode_private_key};

    #[test]
    fn test_decode_sui_bech32() {
        let vectors = [
            ("suiprivkey1qzwant3kaegmjy4qxex93s0jzvemekkjmyv3r2sjwgnv2y479pgsywhveae", "9dd9ae36ee51b912a0364c58c1f21333bcdad2d91911aa127226c512be285102"),
            ("suiprivkey1qrh2sjl88rze74hwjndw3l26dqyz63tea5u9frtwcsqhmfk9vxdlx8cpv0g", "eea84be738c59f56ee94dae8fd5a68082d4579ed38548d6ec4017da6c5619bf3"),
            ("suiprivkey1qzg73qyvfz0wpnyectkl08nrhe4pgnu0vqx8gydu96qx7uj4wyr8gcrjlh3", "91e8808c489ee0cc99c2edf79e63be6a144f8f600c7411bc2e806f7255710674"),
        ];
        for (value, private_key) in vectors {
            let bytes = decode_private_key(&Chain::Sui, value).unwrap();
            assert_eq!(hex::encode(bytes.as_slice()), private_key);
            assert_eq!(encode_private_key(&Chain::Sui, &bytes).unwrap(), value);
            assert!(decode_private_key(&Chain::Aptos, value).is_err());
            assert!(decode_private_key(&Chain::Ethereum, value).is_err());
        }

        let private_key = [7u8; 32];
        let secp256k1_key: Vec<u8> = [1u8].into_iter().chain(private_key).collect();
        let short_key = [SUI_ED25519_FLAG; 32];
        let ed25519_key: Vec<u8> = [SUI_ED25519_FLAG].into_iter().chain(private_key).collect();

        assert!(decode_private_key(&Chain::Sui, &bech32::encode::<Bech32>(SUI_PRIVATE_KEY_HRP, &secp256k1_key).unwrap()).is_err());
        assert!(decode_private_key(&Chain::Sui, &bech32::encode::<Bech32>(SUI_PRIVATE_KEY_HRP, &short_key).unwrap()).is_err());
        assert!(decode_private_key(&Chain::Sui, &bech32::encode::<Bech32m>(SUI_PRIVATE_KEY_HRP, &ed25519_key).unwrap()).is_err());
        assert!(decode_private_key(&Chain::Sui, &bech32::encode::<Bech32>(Hrp::parse_unchecked("sui"), &ed25519_key).unwrap()).is_err());
        assert!(decode_private_key(&Chain::Sui, "suiprivkey1qzwant3kaegmjy4qxex93s0jzvemekkjmyv3r2sjwgnv2y479pgsywhveaf").is_err());
        assert!(decode_private_key(&Chain::Sui, "suiprivkey1qzwant3kaegmjy4qxex93s0jzvemekkjmyv3r2sjwgnv2y479pgs9npcvqt").is_err());
        assert_eq!(
            decode_private_key(&Chain::Sui, "0x9dd9ae36ee51b912a0364c58c1f21333bcdad2d91911aa127226c512be285102").unwrap(),
            decode_private_key(&Chain::Sui, vectors[0].0).unwrap()
        );
    }
}
