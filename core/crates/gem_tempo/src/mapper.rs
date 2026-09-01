use std::error::Error;

use gem_evm::ethereum_address_checksum;
use gem_evm::rpc::model::TransactionReceipt;
use primitives::{AssetId, Chain, Transaction};

use crate::fee::scale_fee_to_token_units;

pub(crate) fn map_transaction(transaction: Transaction, receipt: &TransactionReceipt) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
    let fee_token = receipt.fee_token.as_deref().ok_or("Missing Tempo fee token")?;
    let fee_asset_id = AssetId::from_token(Chain::Tempo, &ethereum_address_checksum(fee_token)?);

    Ok(Transaction {
        fee: scale_fee_to_token_units(receipt.get_fee().into()).try_into()?,
        fee_asset_id,
        ..transaction
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_evm::ethereum_address_checksum;
    use gem_evm::rpc::mapper::EthereumMapper;
    use gem_evm::rpc::model::Transaction as RpcTransaction;
    use num_bigint::BigUint;
    use primitives::testkit::json_rpc::load_json_rpc_result;
    use primitives::{
        Chain, TransactionSwapMetadata, TransactionType,
        asset_constants::{TEMPO_BRIDGED_USDC_ASSET_ID, TEMPO_BRIDGED_USDC_TOKEN_ID, TEMPO_PATHUSD_ASSET_ID, TEMPO_PATHUSD_TOKEN_ID},
    };

    fn map_tempo_transaction(transaction: &RpcTransaction, receipt: &TransactionReceipt) -> Transaction {
        let mapped = EthereumMapper::map_transaction(Chain::Tempo, transaction, receipt, &BigUint::from(1735671600u64)).unwrap();
        map_transaction(mapped, receipt).unwrap()
    }

    #[test]
    fn test_map_transaction_batched_swap() {
        let transaction = load_json_rpc_result::<RpcTransaction>(include_str!("../testdata/tempo_swap_batched_transaction.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../testdata/tempo_swap_batched_transaction_receipt.json"));

        let mapped_transaction = map_tempo_transaction(&transaction, &receipt);

        assert_eq!(mapped_transaction.transaction_type, TransactionType::Swap);
        assert_eq!(mapped_transaction.fee, BigUint::from(595u64));
        assert_eq!(mapped_transaction.fee_asset_id, TEMPO_BRIDGED_USDC_ASSET_ID.clone());

        let metadata: TransactionSwapMetadata = serde_json::from_value(mapped_transaction.metadata.unwrap()).unwrap();
        assert_eq!(metadata.from_asset, TEMPO_BRIDGED_USDC_ASSET_ID.clone());
        assert_eq!(metadata.from_value, "200000");
        assert_eq!(metadata.to_asset, TEMPO_PATHUSD_ASSET_ID.clone());
        assert_eq!(metadata.to_value, "198979");
    }

    #[test]
    fn test_map_transaction_tip20_transfer_identity() {
        let from = crate::testkit::TEMPO_TEST_ADDRESS;
        let to = "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC";
        let pathusd = TEMPO_PATHUSD_TOKEN_ID;

        let pathusd_transaction = RpcTransaction::mock_erc20_transfer(pathusd);
        let mut pathusd_receipt = TransactionReceipt::mock_with_log(gem_evm::rpc::model::Log::mock_erc20_transfer(pathusd, from, to, 1_000_000));
        pathusd_receipt.fee_token = Some(pathusd.to_string());
        let pathusd_transfer = map_tempo_transaction(&pathusd_transaction, &pathusd_receipt);
        assert_eq!(pathusd_transfer.transaction_type, TransactionType::Transfer);
        assert_eq!(pathusd_transfer.asset_id, TEMPO_PATHUSD_ASSET_ID.clone());

        let token_transaction = RpcTransaction::mock_erc20_transfer(TEMPO_BRIDGED_USDC_TOKEN_ID);
        let mut token_receipt = TransactionReceipt::mock_with_log(gem_evm::rpc::model::Log::mock_erc20_transfer(TEMPO_BRIDGED_USDC_TOKEN_ID, from, to, 1_000_000));
        token_receipt.fee_token = Some(TEMPO_BRIDGED_USDC_TOKEN_ID.to_string());
        let token_transfer = map_tempo_transaction(&token_transaction, &token_receipt);
        assert_eq!(token_transfer.transaction_type, TransactionType::Transfer);
        assert_eq!(token_transfer.asset_id, TEMPO_BRIDGED_USDC_ASSET_ID.clone());
        assert_eq!(
            ethereum_address_checksum(&token_transfer.asset_id.token_id.clone().unwrap()).unwrap(),
            TEMPO_BRIDGED_USDC_TOKEN_ID
        );

        token_receipt.fee_token = None;
        let transaction = EthereumMapper::map_transaction(Chain::Tempo, &token_transaction, &token_receipt, &BigUint::from(1735671600u64)).unwrap();
        assert!(map_transaction(transaction, &token_receipt).is_err());
    }
}
