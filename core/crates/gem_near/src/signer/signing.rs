use gem_encoding::encode_base64;
use gem_hash::sha2::sha256;
use primitives::SignerError;
use signer::{ED25519_KEY_TYPE, Ed25519KeyPair};

use super::{models::NearTransaction, serialization::encode_transaction};

pub(super) fn sign(transaction: &NearTransaction, private_key: &[u8]) -> Result<String, SignerError> {
    let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
    let encoded = encode_transaction(transaction, &key_pair.public_key_bytes);
    let signature = key_pair.sign(&sha256(&encoded));

    let mut signed = encoded;
    signed.push(ED25519_KEY_TYPE);
    signed.extend_from_slice(&signature);
    Ok(encode_base64(&signed))
}
