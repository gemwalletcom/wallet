use std::str::FromStr;

use bitcoin::Network as BitcoinNetwork;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use primitives::hex;

use crate::error::LightningError;

const ACCOUNT_PATH: &str = "m/8797555'/1'";

pub struct AccountKey(Xpriv);

impl AccountKey {
    pub fn encode(&self) -> String {
        hex::encode(self.0.encode())
    }

    pub fn decode(value: &str) -> Result<Self, LightningError> {
        let bytes = hex::decode_hex(value).map_err(|error| LightningError::Key(error.to_string()))?;
        Xpriv::decode(&bytes).map(Self).map_err(|error| LightningError::Key(error.to_string()))
    }

    pub(crate) fn xpriv(&self) -> Xpriv {
        self.0
    }
}

impl Drop for AccountKey {
    fn drop(&mut self) {
        self.0.private_key.non_secure_erase();
    }
}

pub fn account_key(seed: &[u8]) -> Result<AccountKey, LightningError> {
    let path = DerivationPath::from_str(ACCOUNT_PATH).map_err(|error| LightningError::Key(error.to_string()))?;
    let mut master = Xpriv::new_master(BitcoinNetwork::Bitcoin, seed).map_err(|error| LightningError::Key(error.to_string()))?;
    let account = master.derive_priv(&Secp256k1::signing_only(), &path);
    master.private_key.non_secure_erase();
    account.map(AccountKey).map_err(|error| LightningError::Key(error.to_string()))
}
