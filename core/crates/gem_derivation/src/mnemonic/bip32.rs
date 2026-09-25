use std::str::FromStr;

use ::bitcoin::Network;
use ::bitcoin::bip32::{DerivationPath, Xpriv};
use zeroize::Zeroizing;

use crate::AccountDerivationError;

pub(super) struct ZeroizedXpriv(pub(super) Xpriv);

impl Drop for ZeroizedXpriv {
    fn drop(&mut self) {
        self.0.private_key.non_secure_erase();
    }
}

pub(super) fn derive_secp256k1_private_key(seed: &[u8], path: &str) -> Result<Zeroizing<Vec<u8>>, AccountDerivationError> {
    let secp = ::bitcoin::secp256k1::Secp256k1::signing_only();
    let master = ZeroizedXpriv(Xpriv::new_master(Network::Bitcoin, seed).map_err(map_bip32_error)?);
    let derivation_path = DerivationPath::from_str(path).map_err(map_bip32_error)?;
    let derived = ZeroizedXpriv(master.0.derive_priv(&secp, &derivation_path).map_err(map_bip32_error)?);
    Ok(Zeroizing::new(derived.0.private_key[..].to_vec()))
}

fn map_bip32_error(error: impl std::fmt::Display) -> AccountDerivationError {
    AccountDerivationError::invalid_input(format!("invalid secp256k1 derivation: {}", error))
}
