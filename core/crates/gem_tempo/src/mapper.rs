use gem_evm::ethereum_address_checksum;
use gem_evm::rpc::model::TransactionReceipt;
use primitives::{AssetId, Chain, Transaction, TransactionSwapMetadata};

use crate::fee::{is_pathusd_contract, scale_fee_to_token_units};

pub(crate) fn map_transaction(transaction: Transaction, receipt: &TransactionReceipt) -> Transaction {
    let fee_asset_id = receipt
        .fee_token
        .as_deref()
        .and_then(|fee_token| ethereum_address_checksum(fee_token).ok())
        .map(|fee_token| map_asset_id(AssetId::from_token(Chain::Tempo, &fee_token)))
        .unwrap_or_else(|| AssetId::from_chain(Chain::Tempo));

    let metadata = match transaction.metadata.clone().map(serde_json::from_value::<TransactionSwapMetadata>) {
        Some(Ok(swap)) => serde_json::to_value(TransactionSwapMetadata {
            from_asset: map_asset_id(swap.from_asset.clone()),
            to_asset: map_asset_id(swap.to_asset.clone()),
            ..swap
        })
        .ok(),
        _ => transaction.metadata.clone(),
    };

    Transaction {
        asset_id: map_asset_id(transaction.asset_id.clone()),
        fee: scale_fee_to_token_units(receipt.get_fee().into()).to_string(),
        fee_asset_id,
        metadata,
        ..transaction
    }
}

pub(crate) fn map_asset_id(asset_id: AssetId) -> AssetId {
    match asset_id.token_id.as_deref() {
        Some(contract_address) if is_pathusd_contract(contract_address) => AssetId::from_chain(asset_id.chain),
        _ => asset_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_evm::ethereum_address_checksum;
    use gem_evm::rpc::mapper::EthereumMapper;
    use gem_evm::rpc::model::Transaction as RpcTransaction;
    use gem_evm::testkit::rpc_mock::{TEMPO_BATCHED_TRANSACTION_JSON, TEMPO_BATCHED_TRANSACTION_RECEIPT_JSON};
    use num_bigint::BigUint;
    use primitives::testkit::json_rpc::load_json_rpc_result;
    use primitives::{
        AssetId, Chain, TransactionType,
        asset_constants::{TEMPO_USDC_ASSET_ID, TEMPO_USDC_TOKEN_ID},
    };

    fn map_tempo_transaction(transaction: &RpcTransaction, receipt: &TransactionReceipt) -> Transaction {
        let mapped = EthereumMapper::map_transaction(Chain::Tempo, transaction, receipt, &BigUint::from(1735671600u64), &[]).unwrap();
        map_transaction(mapped, receipt)
    }

    #[test]
    fn test_map_transaction_batched_swap() {
        let transaction = load_json_rpc_result::<RpcTransaction>(TEMPO_BATCHED_TRANSACTION_JSON);
        let receipt = load_json_rpc_result::<TransactionReceipt>(TEMPO_BATCHED_TRANSACTION_RECEIPT_JSON);

        let mapped_transaction = map_tempo_transaction(&transaction, &receipt);

        assert_eq!(mapped_transaction.transaction_type, TransactionType::Swap);
        assert_eq!(mapped_transaction.fee, "595");
        assert_eq!(mapped_transaction.fee_asset_id, TEMPO_USDC_ASSET_ID.clone());

        let metadata: TransactionSwapMetadata = serde_json::from_value(mapped_transaction.metadata.unwrap()).unwrap();
        assert_eq!(metadata.from_asset, TEMPO_USDC_ASSET_ID.clone());
        assert_eq!(metadata.from_value, "200000");
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Tempo));
        assert_eq!(metadata.to_value, "198979");
    }

    #[test]
    fn test_map_transaction_native_contract_transfer_identity() {
        let from = crate::testkit::TEMPO_TEST_ADDRESS;
        let to = "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC";
        let pathusd = primitives::asset_constants::TEMPO_PATHUSD_TOKEN_ID;

        let native_transaction = RpcTransaction::mock_erc20_transfer(pathusd);
        let native_receipt = TransactionReceipt::mock_with_log(gem_evm::rpc::model::Log::mock_erc20_transfer(pathusd, from, to, 1_000_000));
        let native_transfer = map_tempo_transaction(&native_transaction, &native_receipt);
        assert_eq!(native_transfer.transaction_type, TransactionType::Transfer);
        assert_eq!(native_transfer.asset_id, AssetId::from_chain(Chain::Tempo));

        let token_transaction = RpcTransaction::mock_erc20_transfer(TEMPO_USDC_TOKEN_ID);
        let token_receipt = TransactionReceipt::mock_with_log(gem_evm::rpc::model::Log::mock_erc20_transfer(TEMPO_USDC_TOKEN_ID, from, to, 1_000_000));
        let token_transfer = map_tempo_transaction(&token_transaction, &token_receipt);
        assert_eq!(token_transfer.transaction_type, TransactionType::Transfer);
        assert_eq!(token_transfer.asset_id, TEMPO_USDC_ASSET_ID.clone());
        assert_eq!(ethereum_address_checksum(&token_transfer.asset_id.token_id.clone().unwrap()).unwrap(), TEMPO_USDC_TOKEN_ID);
    }
}
