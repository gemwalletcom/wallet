use std::str::FromStr;

use bitcoin::Network as BitcoinNetwork;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;

use crate::error::LightningError;

const ACCOUNT_PATH: &str = "m/8797555'/1'";

pub struct AccountKey(Xpriv);

impl AccountKey {
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

#[cfg(test)]
mod tests {
    use bitcoin::secp256k1::PublicKey;
    use primitives::hex::decode_hex;

    use super::*;
    use crate::testkit::{ABANDON_SEED_HEX, IDENTITY_PUBLIC_KEY};

    #[test]
    fn test_account_key() {
        let secp = Secp256k1::new();
        let account = account_key(&decode_hex(ABANDON_SEED_HEX).unwrap()).unwrap();
        let identity = account.xpriv().derive_priv(&secp, &DerivationPath::from_str("m/0'").unwrap()).unwrap();

        assert_eq!(PublicKey::from_secret_key(&secp, &identity.private_key).to_string(), IDENTITY_PUBLIC_KEY);
    }
}
