use std::str::FromStr;

use bitcoin::Network as BitcoinNetwork;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;

use crate::error::LightningError;
use crate::model::LightningNetwork;

const MAINNET_ACCOUNT_PATH: &str = "m/8797555'/1'";
const REGTEST_ACCOUNT_PATH: &str = "m/8797555'/0'";

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

pub fn account_key(seed: &[u8], network: LightningNetwork) -> Result<AccountKey, LightningError> {
    let path = match network {
        LightningNetwork::Mainnet => MAINNET_ACCOUNT_PATH,
        LightningNetwork::Regtest => REGTEST_ACCOUNT_PATH,
    };
    let path = DerivationPath::from_str(path).map_err(|error| LightningError::Key(error.to_string()))?;
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
    use crate::testkit::ABANDON_SEED_HEX;

    fn identity_public_key(account: &AccountKey) -> String {
        let secp = Secp256k1::new();
        let identity = account.xpriv().derive_priv(&secp, &DerivationPath::from_str("m/0'").unwrap()).unwrap();
        PublicKey::from_secret_key(&secp, &identity.private_key).to_string()
    }

    #[test]
    fn test_account_key() {
        let seed = decode_hex(ABANDON_SEED_HEX).unwrap();

        assert_eq!(
            identity_public_key(&account_key(&seed, LightningNetwork::Mainnet).unwrap()),
            "0281363910b0dc0015a4a25e758da30f0e28388ea5252c0e3713936f2d4ef7d3d5"
        );
        assert_eq!(
            identity_public_key(&account_key(&seed, LightningNetwork::Regtest).unwrap()),
            "02698b27ac308b275671b3ca25436346469d04a5bba578ae39feba1d65897a6abc"
        );
    }
}
