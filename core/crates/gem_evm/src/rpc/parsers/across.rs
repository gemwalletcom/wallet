use crate::u256::u256_to_biguint;
use alloy_primitives::Address;

use crate::across::{asset::AcrossAsset, deployment::AcrossDeployment, deposit::parse_deposit};
use primitives::{Chain, SwapProvider, Transaction as PrimitivesTransaction, TransactionState, TransactionSwapMetadata};

use super::{ParseContext, ParseContextExt, TransactionParser};

pub struct AcrossParser;

impl TransactionParser<ParseContext<'_>, PrimitivesTransaction> for AcrossParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        let Some(deployment) = AcrossDeployment::deployment_by_chain(context.metadata.chain) else {
            return false;
        };

        context.metadata.receipt.logs.iter().any(|log| log.address.eq_ignore_ascii_case(deployment.spoke_pool))
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let deployment = AcrossDeployment::deployment_by_chain(context.metadata.chain)?;
        let logs = context
            .metadata
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
        let from_asset = AcrossDeployment::supported_asset_for_token(*context.metadata.chain, Address::from_word(relay_data.input_token))?;
        let to_asset = AcrossDeployment::supported_asset_for_token(destination_chain, Address::from_word(relay_data.output_token))?;
        let from_value = u256_to_biguint(&(relay_data.input_amount * AcrossAsset::from_asset(&from_asset)?.scale));
        let to_value = u256_to_biguint(&(relay_data.output_amount * AcrossAsset::from_asset(&to_asset)?.scale));
        let referral_fee = relay_data.referral_fee(&to_asset);
        let metadata = TransactionSwapMetadata::new(from_asset, from_value, to_asset, to_value, SwapProvider::Across).with_referral_fee(referral_fee);
        let depositor = Address::from_word(relay_data.depositor).to_checksum(None);
        let recipient = Address::from_word(relay_data.recipient).to_checksum(None);

        let transaction = context.make_swap_transaction(&depositor, &recipient, &metadata)?;
        Some(match transaction.state {
            TransactionState::Confirmed => PrimitivesTransaction {
                state: TransactionState::InTransit,
                ..transaction
            },
            _ => transaction,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use num_bigint::BigUint;

    use crate::rpc::{
        model::{Transaction, TransactionReceipt},
        parsers::ProtocolParsers,
    };
    use primitives::{
        Chain, SwapProvider, TransactionState, TransactionSwapMetadata, TransactionSwapReferralFee, TransactionType,
        asset_constants::{BASE_USDC_ASSET_ID, ETHEREUM_USDC_ASSET_ID, POLYGON_USDC_ASSET_ID},
        testkit::json_rpc::load_json_rpc_result,
    };

    #[test]
    fn test_parse_across_deposit() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/across_polygon_deposit_transaction.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/across_polygon_deposit_receipt.json"));
        let parsed = ProtocolParsers::map_transaction(&Chain::Polygon, &transaction, &receipt, DateTime::default()).unwrap();
        let metadata = serde_json::from_value::<TransactionSwapMetadata>(parsed.metadata.unwrap()).unwrap();

        assert_eq!(parsed.transaction_type, TransactionType::Swap);
        assert_eq!(parsed.state, TransactionState::InTransit);
        assert_eq!(parsed.from, "0x2A49C84B7173e21f9116B2798735f87531526b36");
        assert_eq!(parsed.to, "0x133243d447026345c2B368d7fFe435dbe3C566Eb");
        assert_eq!(metadata.from_asset, POLYGON_USDC_ASSET_ID.clone());
        assert_eq!(metadata.from_value, BigUint::from(10500000u64));
        assert_eq!(metadata.to_asset, BASE_USDC_ASSET_ID.clone());
        assert_eq!(metadata.to_value, BigUint::from(10500000u64));
        assert_eq!(metadata.provider, Some(SwapProvider::Across.id().to_string()));
        assert_eq!(metadata.referral_fee, None);
    }

    #[test]
    fn test_parse_across_arc_deposit_scales_native_usdc() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/across_arc_deposit_transaction.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/across_arc_deposit_receipt.json"));
        let parsed = ProtocolParsers::map_transaction(&Chain::Arc, &transaction, &receipt, DateTime::default()).unwrap();
        let metadata = serde_json::from_value::<TransactionSwapMetadata>(parsed.metadata.unwrap()).unwrap();

        assert_eq!(metadata.from_asset, Chain::Arc.as_asset_id());
        assert_eq!(metadata.from_value, BigUint::from(5_000_000_000_000_000_000u64));
        assert_eq!(metadata.to_asset, ETHEREUM_USDC_ASSET_ID.clone());
        assert_eq!(metadata.to_value, BigUint::from(4_984_358u64));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: ETHEREUM_USDC_ASSET_ID.clone(),
                value: BigUint::from(24921u64),
            })
        );
    }
}
