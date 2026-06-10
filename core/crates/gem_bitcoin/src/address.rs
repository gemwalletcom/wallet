use bech32::{Hrp, segwit};
use bitcoin::{Address as BitcoinNativeAddress, CompressedPublicKey, Network, ScriptBuf};
use primitives::{Address as AddressTrait, BitcoinChain, Chain, SignerError};

use crate::hash::hash160;
use crate::models::address::Address as ModelAddress;
use crate::signer::address::{DOGE_P2PKH_PREFIX, LITECOIN_HRP, ZCASH_TRANSPARENT_P2PKH_PREFIX, script_for_address};

#[derive(Debug, Clone)]
pub struct BitcoinAddress {
    chain: BitcoinChain,
    address: String,
    script_pubkey: ScriptBuf,
}

impl BitcoinAddress {
    pub fn try_parse_for_chain(address: &str, chain: BitcoinChain) -> Option<Self> {
        let script_pubkey = script_for_address(chain, address).ok()?.script_pubkey;
        Some(Self {
            chain,
            address: address.to_string(),
            script_pubkey,
        })
    }

    pub fn is_valid_for_chain(address: &str, chain: Chain) -> bool {
        BitcoinChain::from_chain(chain).is_some_and(|chain| Self::try_parse_for_chain(address, chain).is_some())
    }

    pub fn bitcoin_chain(&self) -> BitcoinChain {
        self.chain
    }

    pub fn from_public_key(chain: BitcoinChain, public_key: &[u8]) -> Result<Self, SignerError> {
        let public_key = CompressedPublicKey::from_slice(public_key).map_err(|_| SignerError::invalid_input("invalid Bitcoin public key"))?;
        let public_key_hash = hash160(&public_key.to_bytes());
        let address = match chain {
            BitcoinChain::Bitcoin => BitcoinNativeAddress::p2wpkh(&public_key, Network::Bitcoin).to_string(),
            BitcoinChain::BitcoinCash => bitcoin_cash_address(public_key_hash)?,
            BitcoinChain::Litecoin => segwit_address(LITECOIN_HRP, &public_key_hash)?,
            BitcoinChain::Doge => prefixed_base58_address(&DOGE_P2PKH_PREFIX, &public_key_hash),
            BitcoinChain::Zcash => prefixed_base58_address(&ZCASH_TRANSPARENT_P2PKH_PREFIX, &public_key_hash),
        };
        Self::try_parse_for_chain(&address, chain).ok_or_else(|| SignerError::invalid_input("invalid derived Bitcoin address"))
    }
}

impl AddressTrait for BitcoinAddress {
    fn try_parse(address: &str) -> Option<Self> {
        [
            BitcoinChain::Bitcoin,
            BitcoinChain::BitcoinCash,
            BitcoinChain::Litecoin,
            BitcoinChain::Doge,
            BitcoinChain::Zcash,
        ]
        .into_iter()
        .find_map(|chain| Self::try_parse_for_chain(address, chain))
    }

    fn as_bytes(&self) -> &[u8] {
        self.script_pubkey.as_bytes()
    }

    fn encode(&self) -> String {
        self.address.clone()
    }
}

pub fn validate_address(address: &str, chain: Chain) -> bool {
    BitcoinAddress::is_valid_for_chain(address, chain)
}

fn bitcoin_cash_address(public_key_hash: [u8; 20]) -> Result<String, SignerError> {
    let address = bitcoincash_addr::Address::new(
        public_key_hash.to_vec(),
        bitcoincash_addr::Scheme::CashAddr,
        bitcoincash_addr::HashType::Key,
        bitcoincash_addr::Network::Main,
    )
    .encode()
    .map_err(SignerError::from_display)?;
    Ok(ModelAddress::new(address, Chain::BitcoinCash).short().to_string())
}

fn segwit_address(hrp: &str, public_key_hash: &[u8; 20]) -> Result<String, SignerError> {
    let hrp = Hrp::parse(hrp).map_err(SignerError::from_display)?;
    segwit::encode_v0(hrp, public_key_hash).map_err(SignerError::from_display)
}

fn prefixed_base58_address(prefix: &[u8], public_key_hash: &[u8; 20]) -> String {
    let mut payload = Vec::with_capacity(prefix.len() + public_key_hash.len());
    payload.extend_from_slice(prefix);
    payload.extend_from_slice(public_key_hash);
    bs58::encode(payload).with_check().into_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Address as AddressTrait, BITCOINCASH_PREFIX};

    #[test]
    fn test_validate_address() {
        let bitcoin = BitcoinAddress::mock();
        let bitcoin_cash = BitcoinAddress::mock_with_chain(BitcoinChain::BitcoinCash);
        let litecoin = BitcoinAddress::mock_with_chain(BitcoinChain::Litecoin);
        let doge = BitcoinAddress::mock_with_chain(BitcoinChain::Doge);
        let zcash = BitcoinAddress::mock_with_chain(BitcoinChain::Zcash);

        assert!(validate_address(&bitcoin.encode(), Chain::Bitcoin));
        assert!(validate_address(&bitcoin_cash.encode(), Chain::BitcoinCash));
        assert!(validate_address(bitcoin_cash.encode().strip_prefix(BITCOINCASH_PREFIX).unwrap(), Chain::BitcoinCash));
        assert!(validate_address(&litecoin.encode(), Chain::Litecoin));
        assert!(validate_address(&doge.encode(), Chain::Doge));
        assert!(validate_address(&zcash.encode(), Chain::Zcash));
        assert!(!validate_address(&bitcoin.encode(), Chain::Litecoin));
        assert!(!validate_address("invalid", Chain::Bitcoin));

        let parsed = BitcoinAddress::try_parse_for_chain(&bitcoin.encode(), BitcoinChain::Bitcoin).unwrap();
        assert_eq!(parsed.bitcoin_chain().get_chain(), Chain::Bitcoin);
        assert_eq!(parsed.encode(), bitcoin.encode());
        assert_eq!(hex::encode(parsed.as_bytes()), "0014751e76e8199196d454941c45d1b3a323f1433bd6");
    }
}
