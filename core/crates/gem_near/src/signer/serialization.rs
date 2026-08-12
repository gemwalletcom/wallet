use signer::ED25519_KEY_TYPE;

use super::models::{NearAction, NearTransaction};

const FUNCTION_CALL_ACTION: u8 = 2;
const TRANSFER_ACTION: u8 = 3;

pub(super) fn encode_transaction(transaction: &NearTransaction, public_key: &[u8; 32]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(192);
    write_bytes(&mut buffer, transaction.signer_id.as_bytes());
    buffer.push(ED25519_KEY_TYPE);
    buffer.extend_from_slice(public_key);
    buffer.extend_from_slice(&transaction.nonce.to_le_bytes());
    write_bytes(&mut buffer, transaction.receiver_id.as_bytes());
    buffer.extend_from_slice(&transaction.block_hash);
    buffer.extend_from_slice(&(transaction.actions.len() as u32).to_le_bytes());

    for action in &transaction.actions {
        match action {
            NearAction::Transfer { deposit } => {
                buffer.push(TRANSFER_ACTION);
                buffer.extend_from_slice(&deposit.to_le_bytes());
            }
            NearAction::FunctionCall { method_name, args, gas, deposit } => {
                buffer.push(FUNCTION_CALL_ACTION);
                write_bytes(&mut buffer, method_name.as_bytes());
                write_bytes(&mut buffer, args);
                buffer.extend_from_slice(&gas.to_le_bytes());
                buffer.extend_from_slice(&deposit.to_le_bytes());
            }
        }
    }
    buffer
}

fn write_bytes(buffer: &mut Vec<u8>, value: &[u8]) {
    buffer.extend_from_slice(&(value.len() as u32).to_le_bytes());
    buffer.extend_from_slice(value);
}
