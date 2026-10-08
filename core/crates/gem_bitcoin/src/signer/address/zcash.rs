use bech32::{Bech32m, Hrp, primitives::decode::CheckedHrpstring};
use primitives::SignerError;

use super::script::{AddressScript, LockingScript, p2pkh_script, p2sh_script};
use crate::hash::hash20;

// Zcash mainnet transparent address version bytes: t1 for P2PKH, t3 for P2SH.
pub(crate) const TRANSPARENT_P2PKH_PREFIX: [u8; 2] = [0x1c, 0xb8];
pub(crate) const TRANSPARENT_P2SH_PREFIX: [u8; 2] = [0x1c, 0xbd];

const TEX_HRP: Hrp = Hrp::parse_unchecked("tex");

pub(super) fn script(address: &str) -> Result<AddressScript, SignerError> {
    if let Ok(decoded) = CheckedHrpstring::new::<Bech32m>(address)
        && decoded.hrp() == TEX_HRP
    {
        let hash = hash20(&decoded.byte_iter().collect::<Vec<_>>())?;
        return Ok(AddressScript::new(p2pkh_script(hash), LockingScript::P2pkh));
    }
    let payload = bs58::decode(address).with_check(None).into_vec().map_err(SignerError::from_display)?;
    if payload.len() != 22 {
        return Err(SignerError::from_display("invalid Zcash address"));
    }
    let hash = hash20(&payload[2..])?;
    match [payload[0], payload[1]] {
        TRANSPARENT_P2PKH_PREFIX => Ok(AddressScript::new(p2pkh_script(hash), LockingScript::P2pkh)),
        TRANSPARENT_P2SH_PREFIX => Ok(AddressScript::new(p2sh_script(hash), LockingScript::P2sh)),
        _ => Err(SignerError::from_display("unsupported Zcash address version")),
    }
}

#[cfg(test)]
mod tests {
    use bech32::{Bech32, Bech32m, Hrp};
    use primitives::testkit::zcash_mock::{TEST_ZCASH_TEX_ADDRESS, TEST_ZCASH_TRANSPARENT_ADDRESS};

    use super::*;

    #[test]
    fn test_script_tex() {
        let expected = script(TEST_ZCASH_TRANSPARENT_ADDRESS).unwrap();
        for address in [TEST_ZCASH_TEX_ADDRESS.to_string(), TEST_ZCASH_TEX_ADDRESS.to_uppercase()] {
            let result = script(&address).unwrap();
            assert_eq!(result.script_pubkey, expected.script_pubkey);
            assert_eq!(result.locking_script, LockingScript::P2pkh);
        }
    }

    #[test]
    fn test_script_rejects_invalid_tex() {
        let addresses = [
            bech32::encode::<Bech32>(TEX_HRP, &[0; 20]).unwrap(),
            bech32::encode::<Bech32m>(Hrp::parse("textest").unwrap(), &[0; 20]).unwrap(),
            bech32::encode::<Bech32m>(Hrp::parse("tex1other").unwrap(), &[0; 20]).unwrap(),
            bech32::encode::<Bech32m>(TEX_HRP, &[0; 19]).unwrap(),
            bech32::encode::<Bech32m>(TEX_HRP, &[0; 21]).unwrap(),
            "tex1s2rt77ggv6q989lr49rkgzmh5slsksa9khdgtq".to_string(),
            "tEX1S2RT77GGV6Q989LR49RKGZMH5SLSKSA9KHDGTE".to_string(),
        ];
        for address in addresses {
            assert!(script(&address).is_err(), "{address}");
        }
    }
}
