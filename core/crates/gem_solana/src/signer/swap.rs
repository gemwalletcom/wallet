use crate::decode_transaction;
use gem_encoding::encode_base64;
use num_traits::ToPrimitive;
use primitives::{SignerError, SignerInput, TransactionFee};
use solana_primitives::sign_message as sign_solana_message;

pub(crate) fn sign(input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
    let swap_data = input.input_type.get_swap_data()?;
    let transaction_base64 = &swap_data.data.data;

    let unit_price = input.fee.unit_price_u64()?;
    let quote_compute_unit_limit = swap_data
        .data
        .gas_limit
        .as_ref()
        .map(|_| swap_data.data.gas_limit_as_u32())
        .transpose()
        .map_err(SignerError::invalid_input)?;

    Ok(vec![sign_transaction(transaction_base64, private_key, unit_price, quote_compute_unit_limit, &input.fee)?])
}

fn sign_transaction(transaction_base64: &str, private_key: &[u8], unit_price: u64, quote_compute_unit_limit: Option<u32>, fee: &TransactionFee) -> Result<String, SignerError> {
    let mut transaction = decode_transaction(transaction_base64).map_err(SignerError::invalid_input)?;

    if transaction.signatures().len() <= 1 {
        let compute_unit_limit = match quote_compute_unit_limit.or(transaction.get_compute_unit_limit()) {
            Some(compute_unit_limit) => Some(compute_unit_limit),
            None => {
                let compute_unit_limit = fee.gas_limit.to_u32().ok_or_else(|| SignerError::invalid_input("invalid gas limit"))?;
                (compute_unit_limit > 0).then_some(compute_unit_limit)
            }
        };
        if unit_price > 0 {
            transaction
                .set_compute_unit_price(unit_price)
                .map_err(|e| SignerError::invalid_input(format!("set compute unit price: {e}")))?;
        }
        if let Some(compute_unit_limit) = compute_unit_limit.filter(|compute_unit_limit| *compute_unit_limit > 0) {
            transaction
                .set_compute_unit_limit(compute_unit_limit)
                .map_err(|e| SignerError::invalid_input(format!("set compute unit limit: {e}")))?;
        }
    }

    let message_bytes = transaction.serialize_message().map_err(|e| SignerError::signing_error(format!("serialize message: {e}")))?;
    let sig = sign_solana_message(private_key, &message_bytes).map_err(|e| SignerError::signing_error(format!("sign: {e}")))?;

    let sigs = transaction.signatures_mut();
    if sigs.is_empty() {
        sigs.push(sig);
    } else {
        sigs[0] = sig;
    }

    let bytes = transaction.serialize().map_err(|e| SignerError::signing_error(format!("serialize transaction: {e}")))?;
    Ok(encode_base64(&bytes))
}

#[cfg(test)]
mod tests {
    use crate::{
        DEFAULT_SWAP_COMPUTE_UNIT_LIMIT,
        signer::{SolanaChainSigner, testkit::SINGLE_SIG_TX},
    };
    use primitives::swap::SwapData;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{Asset, AssetId, Chain, ChainSigner, GasPriceType, SignerInput, SwapProvider, TransactionFee, TransactionInputType, TransactionLoadInput};

    #[test]
    fn test_sign_swap_without_quote_gas_limit_uses_embedded_limit() {
        let signer = SolanaChainSigner;
        let original_limit = crate::decode_transaction(SINGLE_SIG_TX).unwrap().get_compute_unit_limit();
        let swap_data = SwapData::mock_with_provider_data(SwapProvider::Jupiter, SINGLE_SIG_TX, None);
        let input_type = TransactionInputType::Swap(Asset::mock_sol(), Asset::mock_spl_token(), swap_data);
        let input = TransactionLoadInput::mock_with_input_type(input_type);
        let fee = TransactionFee::new_gas_price_type(
            GasPriceType::solana(5_000u64, 0u64, 0u64),
            5_000u64.into(),
            1u64.into(),
            Default::default(),
            AssetId::from_chain(Chain::Solana),
        );
        let input = SignerInput::new(input, fee);

        let result = signer.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap();

        let signed_transaction = crate::decode_transaction(&result[0]).unwrap();
        assert_eq!(signed_transaction.get_compute_unit_limit(), original_limit);
        assert_ne!(signed_transaction.signatures()[0].as_bytes(), &[0u8; 64]);
    }

    #[test]
    fn test_sign_swap_prefers_quote_gas_limit() {
        let signer = SolanaChainSigner;
        let compute_unit_limit = DEFAULT_SWAP_COMPUTE_UNIT_LIMIT.to_string();
        let swap_data = SwapData::mock_with_provider_data(SwapProvider::Jupiter, SINGLE_SIG_TX, Some(&compute_unit_limit));
        let input_type = TransactionInputType::Swap(Asset::mock_sol(), Asset::mock_spl_token(), swap_data);
        let input = TransactionLoadInput::mock_with_input_type(input_type);
        let fee = TransactionFee::new_gas_price_type(
            GasPriceType::solana(5_000u64, 0u64, 0u64),
            5_000u64.into(),
            1u64.into(),
            Default::default(),
            AssetId::from_chain(Chain::Solana),
        );
        let input = SignerInput::new(input, fee);

        let result = signer.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap();

        let signed_transaction = crate::decode_transaction(&result[0]).unwrap();
        assert_eq!(signed_transaction.get_compute_unit_limit(), Some(DEFAULT_SWAP_COMPUTE_UNIT_LIMIT));
    }
}
