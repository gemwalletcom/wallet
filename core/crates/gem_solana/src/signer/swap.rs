use gem_encoding::encode_base64;
use num_traits::ToPrimitive;
use primitives::{SignerError, SignerInput, TransactionFee};

use super::transaction::sign_transaction;
use crate::decode_transaction;

pub(crate) fn sign(input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
    let swap_data = input.input_type.get_swap_data()?;
    let transaction_base64 = &swap_data.data.data;

    let quote_gas_limit = swap_data
        .data
        .gas_limit
        .as_ref()
        .map(|_| swap_data.data.gas_limit_as_u32())
        .transpose()
        .map_err(SignerError::invalid_input)?;

    Ok(vec![sign_swap_transaction(transaction_base64, private_key, quote_gas_limit, &input.fee)?])
}

fn sign_swap_transaction(transaction_base64: &str, private_key: &[u8], quote_gas_limit: Option<u32>, fee: &TransactionFee) -> Result<String, SignerError> {
    let mut transaction = decode_transaction(transaction_base64).map_err(SignerError::invalid_input)?;

    if transaction.signatures().len() <= 1 {
        let gas_limit = match quote_gas_limit.or(transaction.get_compute_unit_limit()) {
            Some(gas_limit) => Some(gas_limit),
            None => {
                let gas_limit = fee.gas_limit.to_u32().ok_or_else(|| SignerError::invalid_input("invalid gas limit"))?;
                (gas_limit > 0).then_some(gas_limit)
            }
        };
        if let Some(config) = transaction.transaction_config_mut() {
            config.priority_fee = Some(fee.priority_fee_u64()?);
        } else {
            let unit_price = fee.unit_price_u64()?;
            if unit_price > 0 && !transaction.set_compute_unit_price(unit_price) {
                return Err(SignerError::invalid_input("Solana swap transaction has no compute unit price instruction to replace"));
            }
        }
        if let Some(gas_limit) = gas_limit.filter(|gas_limit| *gas_limit > 0) {
            transaction.set_compute_unit_limit(gas_limit);
        }
    }

    sign_transaction(&mut transaction, private_key)?;

    let bytes = transaction.serialize().map_err(|e| SignerError::signing_error(format!("serialize transaction: {e}")))?;
    Ok(encode_base64(&bytes))
}

#[cfg(test)]
mod tests {
    use crate::SignatureBytes;
    use crate::signer::{
        SolanaChainSigner,
        testkit::{SINGLE_SIG_TX, transaction_with_wallet_fee_payer},
    };
    use crate::testkit::{mock_legacy_transaction, mock_v1_transaction};
    use gem_encoding::encode_base64;
    use primitives::swap::SwapData;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{Asset, AssetId, Chain, ChainSigner, GasPriceType, SignerInput, SwapProvider, TransactionFee, TransactionInputType, TransactionLoadInput};

    fn swap_input(encoded: &str, gas_limit: Option<&str>, priority_fee: u64, unit_price: u64) -> SignerInput {
        let input_type = TransactionInputType::Swap {
            from_asset: Asset::mock_sol(),
            to_asset: Asset::mock_spl_token(),
            swap_data: SwapData::mock_with_provider_data(SwapProvider::Jupiter, encoded, gas_limit),
        };
        let fee = TransactionFee::new_gas_price_type(
            GasPriceType::solana(5_000u64, priority_fee, unit_price),
            5_000u64.into(),
            1u64.into(),
            Default::default(),
            AssetId::from_chain(Chain::Solana),
        );
        SignerInput::new(TransactionLoadInput::mock_with_input_type(input_type), fee)
    }

    #[test]
    fn test_sign_swap_without_quote_gas_limit_uses_embedded_limit() {
        let encoded = transaction_with_wallet_fee_payer(SINGLE_SIG_TX);
        let original_limit = crate::decode_transaction(&encoded).unwrap().get_compute_unit_limit();

        let result = SolanaChainSigner.sign_swap(&swap_input(&encoded, None, 0, 0), &TEST_PRIVATE_KEY).unwrap();

        let signed_transaction = crate::decode_transaction(&result[0]).unwrap();
        assert_eq!(signed_transaction.get_compute_unit_limit(), original_limit);
        assert_ne!(signed_transaction.signatures()[0].as_bytes(), &[0u8; 64]);
    }

    #[test]
    fn test_sign_swap_prefers_quote_gas_limit() {
        let gas_limit = crate::DEFAULT_SWAP_GAS_LIMIT.to_string();
        let input = swap_input(&transaction_with_wallet_fee_payer(SINGLE_SIG_TX), Some(&gas_limit), 0, 0);

        let result = SolanaChainSigner.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap();

        let signed_transaction = crate::decode_transaction(&result[0]).unwrap();
        assert_eq!(signed_transaction.get_compute_unit_limit(), Some(crate::DEFAULT_SWAP_GAS_LIMIT));
    }

    #[test]
    fn test_sign_swap_v1_uses_absolute_priority_fee_and_quote_gas_limit() {
        let transaction = mock_v1_transaction(1, 0);
        let original_config = *transaction.transaction_config().unwrap();
        let encoded = encode_base64(&transaction.serialize().unwrap());
        let gas_limit = crate::DEFAULT_SWAP_GAS_LIMIT.to_string();

        let result = SolanaChainSigner
            .sign_swap(&swap_input(&encoded, Some(&gas_limit), 12_345, 999_999), &TEST_PRIVATE_KEY)
            .unwrap();
        let signed = crate::decode_transaction(&result[0]).unwrap();

        assert_eq!(signed.get_priority_fee(), Some(12_345));
        assert_eq!(signed.get_compute_unit_limit(), Some(crate::DEFAULT_SWAP_GAS_LIMIT));
        assert_eq!(signed.get_compute_unit_price(), None);
        assert_eq!(
            signed.transaction_config().unwrap().loaded_accounts_data_size_limit,
            original_config.loaded_accounts_data_size_limit
        );
        assert_eq!(signed.transaction_config().unwrap().heap_size, original_config.heap_size);

        let result = SolanaChainSigner.sign_swap(&swap_input(&encoded, Some(&gas_limit), 0, 999_999), &TEST_PRIVATE_KEY).unwrap();
        let signed = crate::decode_transaction(&result[0]).unwrap();
        assert_eq!(signed.get_priority_fee(), Some(0));
    }

    #[test]
    fn test_sign_swap_fills_the_wallet_signer_slot() {
        let transaction = mock_v1_transaction(2, 1);
        let original_config = *transaction.transaction_config().unwrap();
        let message_bytes = transaction.serialize_message().unwrap();
        let fee_payer_signature = transaction.signatures()[0];
        let encoded = encode_base64(&transaction.serialize().unwrap());
        let gas_limit = crate::DEFAULT_SWAP_GAS_LIMIT.to_string();

        let result = SolanaChainSigner
            .sign_swap(&swap_input(&encoded, Some(&gas_limit), 12_345, 999_999), &TEST_PRIVATE_KEY)
            .unwrap();
        let signed = crate::decode_transaction(&result[0]).unwrap();
        let expected_signature = crate::signer::sign_message(&TEST_PRIVATE_KEY, &message_bytes).unwrap();

        assert_eq!(*signed.transaction_config().unwrap(), original_config);
        assert_eq!(signed.signatures(), &[fee_payer_signature, expected_signature]);
    }

    #[test]
    fn test_sign_swap_rejects_a_missing_compute_unit_price() {
        let mut transaction = mock_legacy_transaction();
        transaction.add_signature(SignatureBytes::new([0; 64]));
        let encoded = encode_base64(&transaction.serialize().unwrap());

        let result = SolanaChainSigner.sign_swap(&swap_input(&encoded, None, 12_345, 999_999), &TEST_PRIVATE_KEY);

        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid input: Solana swap transaction has no compute unit price instruction to replace"
        );
    }
}
