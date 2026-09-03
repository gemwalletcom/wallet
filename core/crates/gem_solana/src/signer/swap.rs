use super::transaction;
use primitives::{SignerError, SignerInput};

pub(crate) fn sign(input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
    let swap_data = input.input_type.get_swap_data()?;
    let transaction_base64 = &swap_data.data.data;
    Ok(vec![sign_transaction(transaction_base64, private_key, &input.fee)?])
}

fn sign_transaction(transaction_base64: &str, private_key: &[u8], fee: &primitives::TransactionFee) -> Result<String, SignerError> {
    transaction::sign_and_encode_transaction(transaction::prepare_with_fee(transaction_base64, fee)?, private_key)
}

#[cfg(test)]
mod tests {
    use crate::signer::{SolanaChainSigner, testkit::SINGLE_SIG_TX};
    use primitives::swap::SwapData;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{Asset, AssetId, Chain, ChainSigner, GasPriceType, SignerInput, SwapProvider, TransactionFee, TransactionInputType, TransactionLoadInput};

    #[test]
    fn test_sign_swap_uses_fee_compute_unit_limit() {
        let signer = SolanaChainSigner;
        let swap_data = SwapData::mock_with_provider_data(SwapProvider::Jupiter, SINGLE_SIG_TX, Some("420000"));
        let input_type = TransactionInputType::Swap(Asset::mock_sol(), Asset::mock_spl_token(), swap_data);
        let input = TransactionLoadInput::mock_with_input_type(input_type);
        let fee = TransactionFee::new_gas_price_type(
            GasPriceType::solana(5_000u64, 0u64, 0u64),
            5_000u64.into(),
            85_002u64.into(),
            Default::default(),
            AssetId::from_chain(Chain::Solana),
        );
        let input = SignerInput::new(input, fee);

        let result = signer.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap();

        let signed_transaction = crate::decode_transaction(&result[0]).unwrap();
        assert_eq!(signed_transaction.get_compute_unit_limit(), Some(85_002));
        assert_ne!(signed_transaction.signatures()[0].as_bytes(), &[0u8; 64]);
    }
}
