use primitives::{ChainSigner, SignerError, SignerInput, TransactionInputType, decode_hex, stake_type::StakeType};

use super::signature::{sign_digest, sign_personal_message};

#[derive(Default)]
pub struct SuiChainSigner;

impl ChainSigner for SuiChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_from_metadata(input, private_key)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_from_metadata(input, private_key)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        sign_from_metadata(input, private_key).map(|signature| vec![signature])
    }

    fn sign_stake(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        match &input.input_type {
            TransactionInputType::Stake(_, stake_type) => match stake_type {
                StakeType::Stake(_) | StakeType::Unstake(_) => {}
                StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) => {
                    return Err(SignerError::SigningError("Sui signer does not support this staking operation yet".to_string()));
                }
                StakeType::Freeze(_) | StakeType::Unfreeze(_) => return Err(SignerError::InvalidInput("Sui does not support freeze operations".to_string())),
            },
            _ => return Err(SignerError::InvalidInput("Expected stake transaction".to_string())),
        }
        sign_from_metadata(input, private_key).map(|signature| vec![signature])
    }

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_from_metadata(input, private_key)
    }

    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        sign_personal_message(message, private_key)
    }
}

fn sign_from_metadata(input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
    let message_bytes = input.metadata.get_message_bytes()?;
    sign_message_bytes(&message_bytes, private_key)
}

fn sign_message_bytes(message: &str, private_key: &[u8]) -> Result<String, SignerError> {
    let (prefix, digest_hex) = message.split_once('_').ok_or_else(|| SignerError::InvalidInput("Invalid Sui digest payload".to_string()))?;

    let digest = decode_hex(digest_hex).map_err(|_| SignerError::InvalidInput("Invalid digest hex for Sui transaction".to_string()))?;

    let signature = sign_digest(&digest, private_key)?;

    Ok(format!("{prefix}_{signature}"))
}
