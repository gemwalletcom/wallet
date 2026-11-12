use gem_hash::blake2::blake2b_256;

use super::Digest;
use crate::SuiError;

const INTENT_TRANSACTION: [u8; 3] = [0, 0, 0];
const INTENT_PERSONAL_MESSAGE: [u8; 3] = [3, 0, 0];

pub fn transaction_signing_digest(transaction_bcs: &[u8]) -> [u8; Digest::LENGTH] {
    blake2b_256(&[INTENT_TRANSACTION.as_slice(), transaction_bcs].concat())
}

pub fn personal_message_signing_digest(message: &[u8]) -> Result<[u8; Digest::LENGTH], SuiError> {
    let message_bcs = bcs::to_bytes(message).map_err(SuiError::from_display)?;
    Ok(blake2b_256(&[INTENT_PERSONAL_MESSAGE.as_slice(), &message_bcs].concat()))
}
