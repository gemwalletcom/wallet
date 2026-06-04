use bitcoin::{
    PublicKey,
    secp256k1::{PublicKey as Secp256k1PublicKey, Secp256k1, SecretKey},
};
use primitives::{BitcoinChain, testkit::signer_mock::TEST_PRIVATE_KEY};

use crate::{
    address::BitcoinAddress,
    signer::address::{UnlockingScript, ZCASH_TRANSPARENT_P2PKH_PREFIX, public_key_hash, script_for_public_key_hash},
};

pub(crate) const TEST_BITCOIN_P2WPKH_ADDRESS: &str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
pub(crate) const TEST_BITCOIN_P2WPKH_HASH: [u8; 20] = [
    0x75, 0x1e, 0x76, 0xe8, 0x19, 0x91, 0x96, 0xd4, 0x54, 0x94, 0x1c, 0x45, 0xd1, 0xb3, 0xa3, 0x23, 0xf1, 0x43, 0x3b, 0xd6,
];

impl BitcoinAddress {
    pub fn mock() -> Self {
        Self::mock_with_chain(BitcoinChain::Bitcoin)
    }

    pub fn mock_with_chain(chain: BitcoinChain) -> Self {
        Self::try_parse_for_chain(&Self::mock_address_with_chain(chain), chain).unwrap()
    }

    pub fn mock_address_with_chain(chain: BitcoinChain) -> String {
        match chain {
            BitcoinChain::Bitcoin => TEST_BITCOIN_P2WPKH_ADDRESS.to_string(),
            BitcoinChain::BitcoinCash => mock_addr_by_hash(chain, [2u8; 20]),
            BitcoinChain::Litecoin => mock_addr_by_hash(chain, [3u8; 20]),
            BitcoinChain::Doge => mock_addr_by_hash(chain, [4u8; 20]),
            BitcoinChain::Zcash => mock_addr_by_hash(chain, [5u8; 20]),
        }
    }
}

pub(crate) fn mock_addr_by_hash(chain: BitcoinChain, hash: [u8; 20]) -> String {
    match chain {
        BitcoinChain::Bitcoin => prefixed_address(&[0], hash),
        BitcoinChain::BitcoinCash => mock_bch_address(hash),
        BitcoinChain::Litecoin => prefixed_address(&[48], hash),
        BitcoinChain::Doge => prefixed_address(&[30], hash),
        BitcoinChain::Zcash => mock_zec_address(hash),
    }
}

pub(crate) fn mock_bch_address(hash: [u8; 20]) -> String {
    bitcoincash_addr::Address::new(
        hash.to_vec(),
        bitcoincash_addr::Scheme::CashAddr,
        bitcoincash_addr::HashType::Key,
        bitcoincash_addr::Network::Main,
    )
    .encode()
    .unwrap()
}

pub(crate) fn mock_zec_address(hash: [u8; 20]) -> String {
    prefixed_address(&ZCASH_TRANSPARENT_P2PKH_PREFIX, hash)
}

pub(crate) fn prefixed_address(prefix: &[u8], hash: [u8; 20]) -> String {
    let mut payload = prefix.to_vec();
    payload.extend(hash);
    bs58::encode(payload).with_check().into_string()
}

pub fn mock_public_key() -> PublicKey {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&TEST_PRIVATE_KEY).unwrap();
    PublicKey::new(Secp256k1PublicKey::from_secret_key(&secp, &secret_key))
}

pub fn mock_sender_address(chain: BitcoinChain) -> String {
    let public_key = mock_public_key();
    mock_addr_by_hash(chain, public_key_hash(&public_key.to_bytes()))
}

pub fn mock_destination_address(chain: BitcoinChain) -> String {
    let hash = match chain {
        BitcoinChain::Bitcoin => public_key_hash(&mock_public_key().to_bytes()),
        BitcoinChain::BitcoinCash | BitcoinChain::Litecoin | BitcoinChain::Doge => [2u8; 20],
        BitcoinChain::Zcash => [3u8; 20],
    };
    mock_addr_by_hash(chain, hash)
}

pub fn mock_p2wpkh_address() -> String {
    let hash = public_key_hash(&mock_public_key().to_bytes());
    let script_pubkey = script_for_public_key_hash(UnlockingScript::P2wpkh, hash);
    bitcoin::Address::from_script(&script_pubkey, bitcoin::Network::Bitcoin).unwrap().to_string()
}
