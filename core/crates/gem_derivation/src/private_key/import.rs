use primitives::{Account, Chain, hex};
use signer::{decode_private_key, supports_private_key_import};
use zeroize::Zeroizing;

use crate::AccountDerivationError;

use super::{
    address::{address_from_private_key, address_from_public_key, public_key_from_private_key},
    path::default_derivation_path,
};

pub struct ImportedPrivateKeyAccount {
    pub account: Account,
    pub private_key: Zeroizing<Vec<u8>>,
}

pub fn import_account_from_private_key(value: &str, chain: Chain) -> Result<ImportedPrivateKeyAccount, AccountDerivationError> {
    let private_key = decode_supported_private_key(value, chain)?;
    let account = derive_account_from_private_key(&private_key, chain)?;
    Ok(ImportedPrivateKeyAccount { account, private_key })
}

pub fn derive_account_from_private_key_value(value: &str, chain: Chain) -> Result<Account, AccountDerivationError> {
    let private_key = decode_supported_private_key(value, chain)?;
    derive_account_from_private_key(&private_key, chain)
}

fn decode_supported_private_key(value: &str, chain: Chain) -> Result<Zeroizing<Vec<u8>>, AccountDerivationError> {
    if !supports_private_key_import(&chain) {
        return Err(AccountDerivationError::InvalidPrivateKey);
    }
    decode_private_key(&chain, value).map_err(|_| AccountDerivationError::InvalidPrivateKey)
}

pub(crate) fn derive_account_from_private_key(private_key: &[u8], chain: Chain) -> Result<Account, AccountDerivationError> {
    let public_key = public_key_from_private_key(private_key, chain)?;
    let address = match &public_key {
        Some(public_key) => address_from_public_key(public_key, chain)?,
        None => address_from_private_key(private_key, chain)?,
    };
    Ok(Account {
        chain,
        address,
        derivation_path: default_derivation_path(chain).to_string(),
        extended_public_key: public_key.map(hex::encode),
    })
}
