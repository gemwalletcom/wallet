use std::collections::HashMap;

use num_bigint::{BigInt, BigUint};

use crate::{
    address::ethereum_address_from_topic,
    ethereum_address_checksum,
    rpc::mapper::TRANSFER_TOPIC,
    uniswap::deployment::{UniversalRouterAbi, v3::get_pancakeswap_router_deployment_by_chain},
};
use primitives::{AssetId, SwapProvider, Transaction as PrimitivesTransaction, TransactionSwapMetadata, decode_hex};

use super::{EVENT_WORD_SIZE, ParseContext, ParseContextExt, TransactionParser, ethereum_value_from_log_data, universal_router::decode_execute_swap};

pub struct PancakeSwapParser;

impl TransactionParser<ParseContext<'_>, PrimitivesTransaction> for PancakeSwapParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        context
            .transaction
            .to
            .as_ref()
            .is_some_and(|to| get_pancakeswap_router_deployment_by_chain(context.metadata.chain).is_some_and(|deployment| deployment.universal_router.eq_ignore_ascii_case(to)))
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let metadata = Self::try_map_transfer_swap(context).or_else(|| Self::try_map_command_swap(context))?;
        context.make_swap_transaction(&context.transaction.from, &context.transaction.from, &metadata)
    }
}

impl PancakeSwapParser {
    fn provider() -> String {
        SwapProvider::PancakeswapV3.id().to_string()
    }

    fn try_map_command_swap(context: &ParseContext<'_>) -> Option<TransactionSwapMetadata> {
        let input_bytes = decode_hex(&context.transaction.input).ok()?;
        decode_execute_swap(
            context.metadata.chain,
            UniversalRouterAbi::V2,
            &Self::provider(),
            &context.transaction.from,
            &input_bytes,
            context.metadata.receipt,
        )
    }

    fn try_map_transfer_swap(context: &ParseContext<'_>) -> Option<TransactionSwapMetadata> {
        let from = ethereum_address_checksum(&context.transaction.from).ok()?;
        let net_by_token = Self::net_erc20_transfers(&from, context);

        let outgoing: Vec<_> = net_by_token.iter().filter(|(_, v)| **v < BigInt::from(0)).collect();
        let incoming: Vec<_> = net_by_token.iter().filter(|(_, v)| **v > BigInt::from(0)).collect();
        let has_native_value = context.transaction.value > BigUint::from(0u8);

        match (has_native_value, outgoing.as_slice(), incoming.as_slice()) {
            (_, [(out_token, out_value)], [(in_token, in_value)]) if out_token != in_token => Some(TransactionSwapMetadata {
                from_asset: AssetId::from_token(*context.metadata.chain, out_token),
                from_value: (-(*out_value).clone()).magnitude().clone(),
                to_asset: AssetId::from_token(*context.metadata.chain, in_token),
                to_value: (*in_value).magnitude().clone(),
                provider: Some(Self::provider()),
            }),
            (true, [], [(in_token, in_value)]) => Some(TransactionSwapMetadata {
                from_asset: AssetId::from_chain(*context.metadata.chain),
                from_value: context.transaction.value.clone(),
                to_asset: AssetId::from_token(*context.metadata.chain, in_token),
                to_value: (*in_value).magnitude().clone(),
                provider: Some(Self::provider()),
            }),
            _ => None,
        }
    }

    fn net_erc20_transfers(user: &str, context: &ParseContext<'_>) -> HashMap<String, BigInt> {
        let mut net_by_token: HashMap<String, BigInt> = HashMap::new();

        for log in &context.metadata.receipt.logs {
            if log.topics.len() != 3 || log.topics.first().is_none_or(|t| t != TRANSFER_TOPIC) {
                continue;
            }
            let Some(token) = ethereum_address_checksum(&log.address).ok() else {
                continue;
            };
            let (Some(log_from), Some(log_to)) = (
                log.topics.get(1).and_then(|t| ethereum_address_from_topic(t)),
                log.topics.get(2).and_then(|t| ethereum_address_from_topic(t)),
            ) else {
                continue;
            };
            let Some(value) = ethereum_value_from_log_data(&log.data, 0, EVENT_WORD_SIZE) else {
                continue;
            };
            let value = BigInt::from(value);

            if log_from == user {
                *net_by_token.entry(token.clone()).or_default() -= value.clone();
            }
            if log_to == user {
                *net_by_token.entry(token).or_default() += value;
            }
        }

        net_by_token
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::{Transaction, TransactionReceipt};
    use crate::rpc::parsers::ProtocolParsers;
    use chrono::DateTime;
    use primitives::{Chain, SwapProvider, TransactionState, TransactionType, testkit::json_rpc::load_json_rpc_result};

    fn map_transaction(chain: &Chain, transaction: &Transaction, receipt: &TransactionReceipt) -> PrimitivesTransaction {
        ProtocolParsers::map_transaction(chain, transaction, receipt, DateTime::from_timestamp(1744602456, 0).unwrap()).unwrap()
    }

    #[test]
    fn test_map_pancakeswap_token_to_token_swap() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/pancakeswap_bsc_swap_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/pancakeswap_bsc_swap_tx_receipt.json"));

        let swap_tx = map_transaction(&Chain::SmartChain, &transaction, &receipt);
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.state, TransactionState::Confirmed);
        assert_eq!(swap_tx.from, "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4");
        assert_eq!(swap_tx.to, "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4");
        assert_eq!(swap_tx.contract.unwrap(), "0x1A0A18AC4BECDDbd6389559687d1A73d8927E416");
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::SmartChain));
        assert_eq!(metadata.provider, Some(SwapProvider::PancakeswapV3.id().to_string()));
        assert_eq!(metadata.from_asset, AssetId::from_token(Chain::SmartChain, "0x55d398326f99059fF775485246999027B3197955"));
        assert_eq!(metadata.from_value, BigUint::parse_bytes(b"2000000000000000000", 10).unwrap());
        assert_eq!(metadata.to_asset, AssetId::from_token(Chain::SmartChain, "0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82"));
        assert_eq!(metadata.to_value, BigUint::parse_bytes(b"1273682274195871312", 10).unwrap());
    }

    #[test]
    fn test_map_pancakeswap_native_to_token_swap() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/pancakeswap_bsc_bnb_cake_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/pancakeswap_bsc_bnb_cake_tx_receipt.json"));

        let swap_tx = map_transaction(&Chain::SmartChain, &transaction, &receipt);
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.state, TransactionState::Confirmed);
        assert_eq!(metadata.provider, Some(SwapProvider::PancakeswapV3.id().to_string()));
        assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::SmartChain));
        assert_eq!(metadata.from_value, BigUint::from(500000000000000000u64));
        assert_eq!(metadata.to_asset, AssetId::from_token(Chain::SmartChain, "0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82"));
        assert_eq!(metadata.to_value, BigUint::from(318420568548967828u64));
    }
}
