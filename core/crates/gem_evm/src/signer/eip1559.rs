use alloy_consensus::{SignableTransaction, TxEip1559, transaction::RlpEcdsaEncodableTx};
use alloy_primitives::Signature;
use primitives::SignerError;
use signer::{SignatureScheme, Signer};

pub fn sign_eip1559_tx(transaction: &TxEip1559, private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let signature_hash = transaction.signature_hash();
    let signature = Signer::sign_digest(SignatureScheme::Secp256k1, signature_hash.as_slice(), private_key)?;
    let signature = Signature::try_from(signature.as_slice()).map_err(SignerError::from_display)?;
    let mut encoded = Vec::with_capacity(transaction.eip2718_encoded_length(&signature));
    transaction.eip2718_encode(&signature, &mut encoded);
    Ok(encoded)
}
