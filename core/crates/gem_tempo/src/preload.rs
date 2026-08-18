use primitives::{Asset, AssetId, AssetType, Chain, TransactionInputType, TransactionLoadInput};

use crate::fee::native_token_contract;

/// Tempo has no native value transfers (`CALLVALUE` is always 0): sending the
/// native asset is an ERC-20 `transfer()` on the pathUSD contract. Rewriting the
/// input to a pathUSD token transfer lets the generic EVM load flow build the
/// call, estimate gas, and resolve the fee without any Tempo branches.
pub fn map_native_transfer_input(input: TransactionLoadInput) -> TransactionLoadInput {
    let asset = match &input.input_type {
        TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) if asset.id.is_native() => asset,
        _ => return input,
    };
    let pathusd = Asset::new(
        AssetId::from_token(Chain::Tempo, native_token_contract()),
        asset.name.clone(),
        asset.symbol.clone(),
        asset.decimals,
        AssetType::TIP20,
    );
    let input_type = match &input.input_type {
        TransactionInputType::Deposit(_) => TransactionInputType::Deposit(pathusd),
        _ => TransactionInputType::Transfer(pathusd),
    };
    TransactionLoadInput { input_type, ..input }
}

#[cfg(test)]
mod tests {
    use gem_evm::provider::preload_mapper::map_evm_transaction_params;
    use num_bigint::BigInt;
    use primitives::{EVMChain, asset_constants::TEMPO_PATHUSD_TOKEN_ID, hex, testkit::signer_mock::TEST_EVM_RECIPIENT};

    use super::*;

    #[test]
    fn test_map_native_transfer_input() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let input = map_native_transfer_input(TransactionLoadInput::mock_evm(TransactionInputType::Transfer(Asset::from_chain(Chain::Tempo)), "1000000"));

        let params = map_evm_transaction_params(EVMChain::Tempo, &input)?;
        assert_eq!(params.to, TEMPO_PATHUSD_TOKEN_ID);
        assert_eq!(params.value, BigInt::ZERO);
        assert_eq!(hex::encode(&params.data[..4]), "a9059cbb");
        assert!(hex::encode(&params.data).contains(&TEST_EVM_RECIPIENT[2..].to_lowercase()));

        let token_asset = Asset::mock_tempo_usdc();
        let unchanged = map_native_transfer_input(TransactionLoadInput::mock_evm(TransactionInputType::Transfer(token_asset.clone()), "1000000"));
        assert_eq!(unchanged.input_type.get_asset().id, token_asset.id);

        Ok(())
    }
}
