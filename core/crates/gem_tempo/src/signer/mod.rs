use alloy_primitives::{Bytes, U256};
use gem_evm::encode::encode_erc20_transfer;
use gem_evm::signer::{EvmSigner, TransactionParams, build_eip1559_transaction, sign_and_encode};
use primitives::{SignerError, SignerInput};

use crate::fee::native_token_contract;

pub struct TempoSigner;

impl EvmSigner for TempoSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let params = TransactionParams::from_input(input)?;
        let data = encode_erc20_transfer(&input.destination_address, &input.get_value()?)?;
        sign_and_encode(&build_eip1559_transaction(&params, native_token_contract(), U256::ZERO, Bytes::from(data))?, private_key)
    }

    fn sign_swap_contract(&self, _input: &SignerInput, _private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        Err(SignerError::invalid_input("Tempo swaps require the 0x76 batched-call transaction"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_evm::signer::EvmChainSigner;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{Asset, Chain, ChainSigner};

    #[test]
    fn test_sign_transfer_native_as_erc20() {
        let metadata = primitives::TransactionLoadMetadata::mock_evm(0, 4217);
        let input = SignerInput::mock_evm_with_metadata(primitives::TransactionInputType::Transfer(Asset::from_chain(Chain::Tempo)), "1000000", 65_000, metadata);
        assert_eq!(
            EvmChainSigner::new(Some(Box::new(TempoSigner))).sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap(),
            "02f8b282107980843b9aca008504a817c80082fde89420c000000000000000000000000000000000000080b844a9059cbb0000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cf00000000000000000000000000000000000000000000000000000000000f4240c080a0d28a29e235b9bdd1f046162709dab035b2fb8d1134c3e91c72a5c67d1d9b3f1fa06f6d021eebf706e572f393f432b605e1f270229eabf57cae46980da4f3d3925d"
        );
    }
}
