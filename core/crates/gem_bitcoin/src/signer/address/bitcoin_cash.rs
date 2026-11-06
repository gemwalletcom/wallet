use bech32::{ByteIterExt, Fe32, Fe32IterExt};
use primitives::SignerError;

use super::script::{AddressScript, LockingScript, p2pkh_script, p2sh_script};
use crate::hash::hash20;

const PREFIX: &str = "bitcoincash";
const P2PKH_VERSION: u8 = 0x00;
const P2SH_VERSION: u8 = 0x08;
const LEGACY_P2PKH_VERSIONS: [u8; 1] = [0x00];
const LEGACY_P2SH_VERSIONS: [u8; 1] = [0x05];
const CHECKSUM_LEN: usize = 8;
const GENERATORS: [u64; 5] = [0x98_f2bc_8e61, 0x79_b76d_99e2, 0xf3_3e5f_b3c4, 0xae_2eab_e2a8, 0x1e_4f43_e470];

pub(crate) fn address(hash: [u8; 20]) -> String {
    let payload: Vec<Fe32> = [P2PKH_VERSION].into_iter().chain(hash).bytes_to_fes().collect();
    let checksum = polymod(payload.iter().copied().chain([Fe32::Q; CHECKSUM_LEN]));
    payload.into_iter().chain(checksum.to_be_bytes()[3..].iter().copied().bytes_to_fes()).map(Fe32::to_char).collect()
}

pub(super) fn script(address: &str) -> Result<AddressScript, SignerError> {
    match cashaddr_script(address) {
        Some(script) => Ok(script),
        None => AddressScript::from_prefixed_address(address, &LEGACY_P2PKH_VERSIONS, &LEGACY_P2SH_VERSIONS, None),
    }
}

fn cashaddr_script(address: &str) -> Option<AddressScript> {
    let payload = match address.split_once(':') {
        Some((PREFIX, payload)) => payload,
        Some(_) => return None,
        None => address,
    };
    if payload.chars().any(|c| c.is_ascii_lowercase()) && payload.chars().any(|c| c.is_ascii_uppercase()) {
        return None;
    }
    let values = payload.chars().map(Fe32::from_char).collect::<Result<Vec<_>, _>>().ok()?;
    let data_len = values.len().checked_sub(CHECKSUM_LEN)?;
    if polymod(values.iter().copied()) != 0 {
        return None;
    }
    let bytes: Vec<u8> = values[..data_len].iter().copied().fes_to_bytes().collect();
    let (&version, hash) = bytes.split_first()?;
    let hash = hash20(hash).ok()?;
    match version {
        P2PKH_VERSION => Some(AddressScript::new(p2pkh_script(hash), LockingScript::P2pkh)),
        P2SH_VERSION => Some(AddressScript::new(p2sh_script(hash), LockingScript::P2sh)),
        _ => None,
    }
}

fn polymod(values: impl Iterator<Item = Fe32>) -> u64 {
    let mut checksum = 1u64;
    for value in PREFIX.bytes().map(|byte| byte & 0x1f).chain([0]).chain(values.map(Fe32::to_u8)) {
        let top = checksum >> 35;
        checksum = ((checksum & 0x07_ffff_ffff) << 5) ^ u64::from(value);
        for (index, generator) in GENERATORS.iter().enumerate() {
            if (top >> index) & 1 == 1 {
                checksum ^= generator;
            }
        }
    }
    checksum ^ 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::hex;

    #[test]
    fn test_address() {
        let hash: [u8; 20] = hex::decode("76a04053bda0a88bda5177b86a15c3b29f559873").unwrap().try_into().unwrap();

        assert_eq!(address(hash), "qpm2qsznhks23z7629mms6s4cwef74vcwvy22gdx6a");
    }

    #[test]
    fn test_script() {
        let p2pkh = "76a91476a04053bda0a88bda5177b86a15c3b29f55987388ac";
        let p2sh = "a91476a04053bda0a88bda5177b86a15c3b29f55987387";
        for (address, expected) in [
            ("bitcoincash:qpm2qsznhks23z7629mms6s4cwef74vcwvy22gdx6a", p2pkh),
            ("bitcoincash:QPM2QSZNHKS23Z7629MMS6S4CWEF74VCWVY22GDX6A", p2pkh),
            ("QPM2QSZNHKS23Z7629MMS6S4CWEF74VCWVY22GDX6A", p2pkh),
            ("1BpEi6DfDAUFd7GtittLSdBeYJvcoaVggu", p2pkh),
            ("bitcoincash:ppm2qsznhks23z7629mms6s4cwef74vcwvn0h829pq", p2sh),
            ("3CWFddi6m4ndiGyKqzYvsFYagqDLPVMTzC", p2sh),
        ] {
            assert_eq!(hex::encode(script(address).unwrap().script_pubkey.as_bytes()), expected, "{address}");
        }

        for address in [
            "bitcoincash:qpm2qsznhks23z7629mms6s4cwef74vcwvy22gdx6b",
            "bitcoincash:qpm2qsznhks23z7629mms6s4cwef74vcwvY22GDX6A",
            "bchtest:qpm2qsznhks23z7629mms6s4cwef74vcwvy22gdx6a",
            "BitcoinCash:qpm2qsznhks23z7629mms6s4cwef74vcwvy22gdx6a",
            "BITCOINCASH:QPM2QSZNHKS23Z7629MMS6S4CWEF74VCWVY22GDX6A",
            "bitcoincash:1BpEi6DfDAUFd7GtittLSdBeYJvcoaVggu",
            "bitcoincash:",
            "mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn",
        ] {
            assert!(script(address).is_err(), "{address}");
        }
    }
}
