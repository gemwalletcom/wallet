use alloy_primitives::Address;

use crate::across::{deployment::AcrossDeployment, deposit::parse_deposit};
use primitives::{Chain, SwapProvider, Transaction as PrimitivesTransaction, TransactionSwapMetadata};

use super::{ParseContext, ProtocolParser};

pub struct AcrossParser;

impl ProtocolParser for AcrossParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        let Some(deployment) = AcrossDeployment::deployment_by_chain(context.chain) else {
            return false;
        };

        context.receipt.logs.iter().any(|log| log.address.eq_ignore_ascii_case(deployment.spoke_pool))
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let deployment = AcrossDeployment::deployment_by_chain(context.chain)?;
        let logs = context
            .receipt
            .logs
            .iter()
            .filter(|log| log.address.eq_ignore_ascii_case(deployment.spoke_pool))
            .map(|log| (log.topics.as_slice(), log.data.as_str()));
        let Ok(Some(deposit)) = parse_deposit(logs, u64::from(deployment.chain_id)) else {
            return None;
        };
        let relay_data = &deposit.relay_data;
        let destination_chain = Chain::from_chain_id(deposit.destination_chain_id)?;
        let metadata = TransactionSwapMetadata {
            from_asset: AcrossDeployment::supported_asset_for_token(*context.chain, Address::from_word(relay_data.input_token))?,
            from_value: relay_data.input_amount.to_string(),
            to_asset: AcrossDeployment::supported_asset_for_token(destination_chain, Address::from_word(relay_data.output_token))?,
            to_value: relay_data.output_amount.to_string(),
            provider: Some(SwapProvider::Across.id().to_string()),
        };
        let depositor = Address::from_word(relay_data.depositor).to_checksum(None);
        let recipient = Address::from_word(relay_data.recipient).to_checksum(None);

        context.make_swap_transaction(&depositor, &recipient, &metadata)
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::rpc::{
        model::{Transaction, TransactionReceipt},
        parsers::ProtocolParsers,
    };
    use primitives::{
        Chain, SwapProvider, TransactionSwapMetadata, TransactionType,
        asset_constants::{BASE_USDC_ASSET_ID, POLYGON_USDC_ASSET_ID},
        testkit::json_rpc::load_json_rpc_result,
    };

    #[test]
    fn test_parse_across_deposit() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/across_polygon_deposit_transaction.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/across_polygon_deposit_receipt.json"));
        let parsed = ProtocolParsers::map_transaction(&Chain::Polygon, &transaction, &receipt, None, None, DateTime::default()).unwrap();
        let metadata = serde_json::from_value::<TransactionSwapMetadata>(parsed.metadata.unwrap()).unwrap();

        assert_eq!(parsed.transaction_type, TransactionType::Swap);
        assert_eq!(parsed.from, "0x2A49C84B7173e21f9116B2798735f87531526b36");
        assert_eq!(parsed.to, "0x133243d447026345c2B368d7fFe435dbe3C566Eb");
        assert_eq!(metadata.from_asset, POLYGON_USDC_ASSET_ID.clone());
        assert_eq!(metadata.from_value, "10500000");
        assert_eq!(metadata.to_asset, BASE_USDC_ASSET_ID.clone());
        assert_eq!(metadata.to_value, "10500000");
        assert_eq!(metadata.provider, Some(SwapProvider::Across.id().to_string()));
    }
}
