use alloy_primitives::Address;

use crate::across::{
    deployment::AcrossDeployment,
    deposit::{Deposit, parse_deposit, parse_deposit_call},
};
use primitives::{Transaction as PrimitivesTransaction, TransactionState, decode_hex};

use super::{ParseContext, ParseContextExt, TransactionParser};

pub struct AcrossParser;

impl TransactionParser<ParseContext<'_>, PrimitivesTransaction> for AcrossParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        let Some(deployment) = AcrossDeployment::deployment_by_chain(context.metadata.chain) else {
            return false;
        };

        context.is_to(deployment.spoke_pool) || context.metadata.receipt.logs.iter().any(|log| log.address.eq_ignore_ascii_case(deployment.spoke_pool))
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
        let deposit = parse_deposit(logs, u64::from(deployment.chain_id)).ok().flatten().or_else(|| Self::reverted_deposit(context, u64::from(deployment.chain_id)))?;
        let metadata = deposit.swap_metadata(AcrossDeployment::supported_asset_for_token)?;
        let metadata = match context.metadata.receipt.get_state() {
            TransactionState::Reverted => metadata.with_referral_fee(None),
            _ => metadata,
        };
        let relay_data = &deposit.relay_data;
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

impl AcrossParser {
    fn reverted_deposit(context: &ParseContext<'_>, origin_chain_id: u64) -> Option<Deposit> {
        if context.metadata.receipt.get_state() != TransactionState::Reverted {
            return None;
        }
        parse_deposit_call(&decode_hex(&context.transaction.input).ok()?, origin_chain_id).ok()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy_primitives::{Address, U256};
    use alloy_sol_types::{SolCall, SolValue};
    use chrono::DateTime;
    use num_bigint::BigUint;

    use crate::across::contracts::{
        V3SpokePoolInterface,
        multicall_handler::{Call, Instructions},
    };
    use crate::contracts::IERC20;
    use crate::rpc::{
        model::{Transaction, TransactionReceipt},
        parsers::ProtocolParsers,
    };
    use primitives::{
        Chain, SwapProvider, TransactionState, TransactionSwapMetadata, TransactionSwapReferralFee, TransactionType,
        asset_constants::{BASE_USDC_ASSET_ID, BASE_USDC_TOKEN_ID, ETHEREUM_USDC_ASSET_ID, POLYGON_USDC_ASSET_ID, POLYGON_USDC_TOKEN_ID},
        contract_constants::POLYGON_ACROSS_SPOKE_POOL_CONTRACT,
        decode_hex,
        hex::encode_with_0x,
        swap::EVM_REFERRAL_ADDRESS,
        testkit::json_rpc::load_json_rpc_result,
    };

    #[test]
    fn test_parse_across_deposit() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/across_polygon_deposit_transaction.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/across_polygon_deposit_receipt.json"));
        let parsed = ProtocolParsers::map_transaction(&Chain::Polygon, &transaction, &receipt, DateTime::default()).unwrap();
        let metadata = parsed.swap_metadata().unwrap();

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
        let metadata = parsed.swap_metadata().unwrap();

        assert_eq!(metadata.from_asset, Chain::Arc.as_asset_id());
        assert_eq!(metadata.from_value, BigUint::from(5_000_000_000_000_000_000u64));
        assert_eq!(metadata.to_asset, ETHEREUM_USDC_ASSET_ID.clone());
        assert_eq!(metadata.to_value, BigUint::from(4_959_437u64));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: ETHEREUM_USDC_ASSET_ID.clone(),
                value: BigUint::from(24921u64),
            })
        );
    }

    #[test]
    fn test_parse_across_reverted_deposit() {
        let wallet = Address::from_str(&Transaction::mock().from).unwrap();
        let message = Instructions {
            calls: vec![Call {
                target: Address::from_str(BASE_USDC_TOKEN_ID).unwrap(),
                callData: IERC20::transferCall {
                    to: Address::from_str(EVM_REFERRAL_ADDRESS).unwrap(),
                    value: U256::from(50_000u64),
                }
                .abi_encode()
                .into(),
                value: U256::ZERO,
            }],
            fallbackRecipient: wallet,
        }
        .abi_encode();
        let transaction = Transaction {
            to: Some(POLYGON_ACROSS_SPOKE_POOL_CONTRACT.to_string()),
            input: encode_with_0x(
                &V3SpokePoolInterface::depositV3Call {
                    depositor: wallet,
                    recipient: wallet,
                    inputToken: Address::from_str(POLYGON_USDC_TOKEN_ID).unwrap(),
                    outputToken: Address::from_str(BASE_USDC_TOKEN_ID).unwrap(),
                    inputAmount: U256::from(10_000_000u64),
                    outputAmount: U256::from(9_950_000u64),
                    destinationChainId: U256::from(8453),
                    exclusiveRelayer: Address::ZERO,
                    quoteTimestamp: 1_700_000_000,
                    fillDeadline: 1_700_003_600,
                    exclusivityDeadline: 0,
                    message: message.into(),
                }
                .abi_encode(),
            ),
            ..Transaction::mock()
        };
        let reverted = TransactionReceipt {
            status: "0x0".to_string(),
            ..TransactionReceipt::mock()
        };
        let parsed = ProtocolParsers::map_transaction(&Chain::Polygon, &transaction, &reverted, DateTime::default()).unwrap();

        assert_eq!(parsed.transaction_type, TransactionType::Swap);
        assert_eq!(parsed.state, TransactionState::Reverted);
        assert_eq!(
            parsed.swap_metadata(),
            Some(TransactionSwapMetadata::new(
                POLYGON_USDC_ASSET_ID.clone(),
                BigUint::from(10_000_000u64),
                BASE_USDC_ASSET_ID.clone(),
                BigUint::from(9_900_000u64),
                SwapProvider::Across
            ))
        );
        assert_eq!(ProtocolParsers::map_transaction(&Chain::Polygon, &transaction, &TransactionReceipt::mock(), DateTime::default()), None);

        let unknown_destination = Transaction {
            input: encode_with_0x(
                &V3SpokePoolInterface::depositV3Call {
                    destinationChainId: U256::MAX,
                    ..V3SpokePoolInterface::depositV3Call::abi_decode(&decode_hex(&transaction.input).unwrap()).unwrap()
                }
                .abi_encode(),
            ),
            ..transaction
        };
        assert_eq!(ProtocolParsers::map_transaction(&Chain::Polygon, &unknown_destination, &reverted, DateTime::default()), None);
    }
}
