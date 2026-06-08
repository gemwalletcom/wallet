use std::str::FromStr;

use ::bitcoin::Network;
use ::bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use primitives::{BitcoinChain, Chain};

use crate::AccountDerivationError;
use crate::private_key::default_derivation_path;

const ACCOUNT_DEPTH: usize = 3;
const XPUB_VERSION: u32 = 0x0488_b21e;
const ZPUB_VERSION: u32 = 0x04b2_4746;
const DGUB_VERSION: u32 = 0x02fa_cafd;

pub(super) fn derive_extended_public_key(seed: &[u8], bitcoin_chain: BitcoinChain) -> Result<String, AccountDerivationError> {
    let secp = ::bitcoin::secp256k1::Secp256k1::signing_only();
    let master = Xpriv::new_master(Network::Bitcoin, seed).map_err(map_bitcoin_error)?;
    let chain = bitcoin_chain.get_chain();
    let account_path = account_derivation_path(chain)?;
    let account_private_key = master.derive_priv(&secp, &account_path).map_err(map_bitcoin_error)?;
    let account_public_key = Xpub::from_priv(&secp, &account_private_key);

    Ok(encode_extended_public_key(&account_public_key, extended_public_key_version(bitcoin_chain)))
}

fn account_derivation_path(chain: Chain) -> Result<DerivationPath, AccountDerivationError> {
    let path = DerivationPath::from_str(default_derivation_path(chain)).map_err(map_bitcoin_error)?;
    let account_path = path
        .as_ref()
        .get(..ACCOUNT_DEPTH)
        .ok_or_else(|| AccountDerivationError::invalid_input(format!("invalid Bitcoin account derivation path for {}", chain)))?;
    Ok(DerivationPath::from(account_path))
}

fn encode_extended_public_key(xpub: &Xpub, version: u32) -> String {
    let mut payload = xpub.encode();
    payload[..4].copy_from_slice(&version.to_be_bytes());
    bs58::encode(payload).with_check().into_string()
}

fn extended_public_key_version(chain: BitcoinChain) -> u32 {
    match chain {
        BitcoinChain::Bitcoin | BitcoinChain::Litecoin => ZPUB_VERSION,
        BitcoinChain::BitcoinCash | BitcoinChain::Zcash => XPUB_VERSION,
        BitcoinChain::Doge => DGUB_VERSION,
    }
}

fn map_bitcoin_error(error: impl std::fmt::Display) -> AccountDerivationError {
    AccountDerivationError::invalid_input(format!("invalid Bitcoin account derivation: {}", error))
}
