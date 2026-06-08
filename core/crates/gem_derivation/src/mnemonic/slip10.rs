use gem_crypto::hash::hmac_sha512;
use zeroize::Zeroizing;

use crate::AccountDerivationError;

use super::crypto::{CHAIN_CODE_LEN, DerivedChild, PRIVATE_KEY_LEN, split_left_key, split_right_chain_code};
use super::path::{HARDENED_OFFSET, parse_derivation_path};

const ED25519_DOMAIN: &[u8] = b"ed25519 seed";

pub(super) fn derive_ed25519_private_key(seed: &[u8], path: &str) -> Result<Zeroizing<Vec<u8>>, AccountDerivationError> {
    let components = parse_derivation_path(path).ok_or_else(|| AccountDerivationError::invalid_input(format!("invalid derivation path: {path}")))?;
    let output = hmac_sha512(ED25519_DOMAIN, seed)?;
    let mut private_key = Zeroizing::new(split_left_key(&output));
    let mut chain_code = Zeroizing::new(split_right_chain_code(&output));

    for component in components {
        if !component.hardened {
            return Err(AccountDerivationError::unsupported("ed25519 mnemonic derivation only supports hardened path components"));
        }
        let child = derive_child(&private_key, &chain_code, component.index)?;
        private_key.copy_from_slice(child.private_key.as_slice());
        chain_code.copy_from_slice(child.chain_code.as_slice());
    }

    Ok(Zeroizing::new(private_key.to_vec()))
}

fn derive_child(private_key: &[u8; PRIVATE_KEY_LEN], chain_code: &[u8; CHAIN_CODE_LEN], index: u32) -> Result<DerivedChild, AccountDerivationError> {
    let mut data = Zeroizing::new(Vec::with_capacity(37));
    data.push(0);
    data.extend_from_slice(private_key);
    data.extend_from_slice(&(index | HARDENED_OFFSET).to_be_bytes());

    let output = hmac_sha512(chain_code, data.as_slice())?;
    Ok(DerivedChild {
        private_key: Zeroizing::new(split_left_key(&output)),
        chain_code: Zeroizing::new(split_right_chain_code(&output)),
    })
}
