use gem_evm::signer::EvmSigner;
use primitives::{SignerError, SignerInput};

pub struct TempoSigner;

impl EvmSigner for TempoSigner {
    fn sign_transfer(&self, _input: &SignerInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::invalid_input("Tempo does not support native transfers"))
    }

    fn sign_swap_contract(&self, _input: &SignerInput, _private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        Err(SignerError::invalid_input("Tempo swaps require the 0x76 batched-call transaction"))
    }
}
