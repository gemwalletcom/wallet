mod bitcoin;
mod bitcoin_cash;
mod doge;
mod litecoin;
mod script;
mod zcash;

use primitives::{BitcoinChain, SignerError};

pub(crate) use crate::hash::public_key_hash;
pub(crate) use doge::P2PKH_VERSIONS as DOGE_P2PKH_PREFIX;
pub(crate) use litecoin::HRP as LITECOIN_HRP;
use script::AddressScript;
pub(crate) use script::{UnlockingScript, script_for_public_key_hash};
pub(crate) use zcash::TRANSPARENT_P2PKH_PREFIX as ZCASH_TRANSPARENT_P2PKH_PREFIX;

pub(crate) fn script_for_address(chain: BitcoinChain, address: &str) -> Result<AddressScript, SignerError> {
    match chain {
        BitcoinChain::Bitcoin => bitcoin::script(address),
        BitcoinChain::Litecoin => litecoin::script(address),
        BitcoinChain::Doge => doge::script(address),
        BitcoinChain::BitcoinCash => bitcoin_cash::script(address),
        BitcoinChain::Zcash => zcash::script(address),
    }
}

#[cfg(test)]
mod tests {
    use primitives::BitcoinChain;

    use super::{
        script::{LockingScript, UnlockingScript},
        script_for_address,
    };

    struct AddressScriptCase {
        address: &'static str,
        locking_script: LockingScript,
        unlocking_script: Option<UnlockingScript>,
        script_pubkey: &'static str,
        public_key_hash: Option<&'static str>,
    }

    impl AddressScriptCase {
        fn assert(&self, chain: BitcoinChain) {
            let script = script_for_address(chain, self.address).unwrap();
            assert_eq!(script.locking_script, self.locking_script, "locking_script for {}", self.address);
            assert_eq!(script.unlocking_script(), self.unlocking_script, "unlocking_script for {}", self.address);
            assert_eq!(hex::encode(script.script_pubkey.as_bytes()), self.script_pubkey, "script_pubkey for {}", self.address);
            assert_eq!(
                script.public_key_hash().map(hex::encode).as_deref(),
                self.public_key_hash,
                "public_key_hash for {}",
                self.address
            );
        }
    }

    #[test]
    fn test_script_for_address_bitcoin() {
        let cases = [
            AddressScriptCase {
                address: "1QJVDzdqb1VpbDK7uDeyVXy9mR27CJiyhY",
                locking_script: LockingScript::P2pkh,
                unlocking_script: Some(UnlockingScript::P2pkh),
                script_pubkey: "76a914ff99864ce1a887e00c9c8615210d6267edd7d7a588ac",
                public_key_hash: Some("ff99864ce1a887e00c9c8615210d6267edd7d7a5"),
            },
            AddressScriptCase {
                address: "33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k",
                locking_script: LockingScript::P2sh,
                unlocking_script: None,
                script_pubkey: "a914162c5ea71c0b23f5b9022ef047c4a86470a5b07087",
                public_key_hash: None,
            },
            AddressScriptCase {
                address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
                locking_script: LockingScript::P2wpkh,
                unlocking_script: Some(UnlockingScript::P2wpkh),
                script_pubkey: "0014751e76e8199196d454941c45d1b3a323f1433bd6",
                public_key_hash: Some("751e76e8199196d454941c45d1b3a323f1433bd6"),
            },
            AddressScriptCase {
                address: "bc1qwqdg6squsna38e46795at95yu9atm8azzmyvckulcc7kytlcckxswvvzej",
                locking_script: LockingScript::P2wsh,
                unlocking_script: None,
                script_pubkey: "0020701a8d401c84fb13e6baf169d59684e17abd9fa216c8cc5b9fc63d622ff8c58d",
                public_key_hash: None,
            },
            AddressScriptCase {
                address: "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr",
                locking_script: LockingScript::P2tr,
                unlocking_script: None,
                script_pubkey: "5120a60869f0dbcf1dc659c9cecbaf8050135ea9e8cdc487053f1dc6880949dc684c",
                public_key_hash: None,
            },
        ];
        for case in &cases {
            case.assert(BitcoinChain::Bitcoin);
        }
    }

    #[test]
    fn test_script_for_address_bitcoin_cash() {
        let cases = [
            AddressScriptCase {
                address: "bitcoincash:qp3wjpa3tjlj042z2wv7hahsldgwhwy0rq9sywjpyy",
                locking_script: LockingScript::P2pkh,
                unlocking_script: Some(UnlockingScript::P2pkh),
                script_pubkey: "76a91462e907b15cbf27d5425399ebf6f0fb50ebb88f1888ac",
                public_key_hash: Some("62e907b15cbf27d5425399ebf6f0fb50ebb88f18"),
            },
            AddressScriptCase {
                address: "qp3wjpa3tjlj042z2wv7hahsldgwhwy0rq9sywjpyy",
                locking_script: LockingScript::P2pkh,
                unlocking_script: Some(UnlockingScript::P2pkh),
                script_pubkey: "76a91462e907b15cbf27d5425399ebf6f0fb50ebb88f1888ac",
                public_key_hash: Some("62e907b15cbf27d5425399ebf6f0fb50ebb88f18"),
            },
            AddressScriptCase {
                address: "bitcoincash:pr0662zpd7vr936d83f64u629v886aan7c77r3j5v5",
                locking_script: LockingScript::P2sh,
                unlocking_script: None,
                script_pubkey: "a914dfad28416f9832c74d3c53aaf34a2b0e7d77b3f687",
                public_key_hash: None,
            },
        ];
        for case in &cases {
            case.assert(BitcoinChain::BitcoinCash);
        }
    }

    #[test]
    fn test_script_for_address_litecoin() {
        let cases = [
            AddressScriptCase {
                address: "LMHEFMwRsQ3nHDfb9zZqynLHxjuJ2hgyyW",
                locking_script: LockingScript::P2pkh,
                unlocking_script: Some(UnlockingScript::P2pkh),
                script_pubkey: "76a914168ed7e47426cf09541df4979c6450b3d5a5547088ac",
                public_key_hash: Some("168ed7e47426cf09541df4979c6450b3d5a55470"),
            },
            AddressScriptCase {
                address: "MC2JYMPVWaxqUb9qUkUbjtUwoNMo1tPaLF",
                locking_script: LockingScript::P2sh,
                unlocking_script: None,
                script_pubkey: "a9142d3a59d2d9f68868cbd5d37afb2c0d6c921b2f3187",
                public_key_hash: None,
            },
            AddressScriptCase {
                address: "ltc1qhzjptwpym9afcdjhs7jcz6fd0jma0l0rc0e5yr",
                locking_script: LockingScript::P2wpkh,
                unlocking_script: Some(UnlockingScript::P2wpkh),
                script_pubkey: "0014b8a415b824d97a9c365787a581692d7cb7d7fde3",
                public_key_hash: Some("b8a415b824d97a9c365787a581692d7cb7d7fde3"),
            },
        ];
        for case in &cases {
            case.assert(BitcoinChain::Litecoin);
        }
        // A Bitcoin P2SH address must not parse under Litecoin's network params.
        assert!(script_for_address(BitcoinChain::Litecoin, "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy").is_err());
    }

    #[test]
    fn test_script_for_address_doge() {
        let cases = [
            AddressScriptCase {
                address: "DMKhUaRmnxJXfDxyFguMnMjVdgvnNipFzt",
                locking_script: LockingScript::P2pkh,
                unlocking_script: Some(UnlockingScript::P2pkh),
                script_pubkey: "76a914b18355f0b9c7aa20e9db204825e6275e9a40bc8988ac",
                public_key_hash: Some("b18355f0b9c7aa20e9db204825e6275e9a40bc89"),
            },
            AddressScriptCase {
                address: "A1yb6viUzAcUWftRHT6GpnCwvhXHg4CV1x",
                locking_script: LockingScript::P2sh,
                unlocking_script: None,
                script_pubkey: "a91468a56b88a61df17afc8a0709ec1536a51101881087",
                public_key_hash: None,
            },
        ];
        for case in &cases {
            case.assert(BitcoinChain::Doge);
        }
    }

    #[test]
    fn test_script_for_address_zcash() {
        let cases = [
            AddressScriptCase {
                address: "t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML",
                locking_script: LockingScript::P2pkh,
                unlocking_script: Some(UnlockingScript::P2pkh),
                script_pubkey: "76a9141634f5ff0b8f6603a17570436d6c12a91f4b1fed88ac",
                public_key_hash: Some("1634f5ff0b8f6603a17570436d6c12a91f4b1fed"),
            },
            AddressScriptCase {
                address: "t3Vz22vK5z2LcKEdg16Yv4FFneEL1zg9ojd",
                locking_script: LockingScript::P2sh,
                unlocking_script: None,
                script_pubkey: "a9147d46a730d31f97b1930d3368a967c309bd4d136a87",
                public_key_hash: None,
            },
        ];
        for case in &cases {
            case.assert(BitcoinChain::Zcash);
        }
    }
}
