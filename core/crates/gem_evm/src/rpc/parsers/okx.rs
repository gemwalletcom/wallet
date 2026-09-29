use alloy_primitives::Address;
use num_bigint::BigUint;
use std::str::FromStr;

use crate::{
    address::ethereum_address_from_topic,
    contracts::{IOkxDexRouter, OkxCommission},
    ethereum_address_checksum,
    rpc::{mapper::TRANSFER_TOPIC, model::Log},
    u256::u256_to_biguint,
};
use primitives::{AssetId, SwapProvider, Transaction as PrimitivesTransaction, TransactionSwapMetadata, TransactionSwapReferralFee, contract_constants::EVM_NATIVE_TOKEN_ADDRESS, swap::EVM_REFERRAL_ADDRESS};

use super::{EVENT_WORD_SIZE, ParseContext, ParseContextExt, TransactionParser, ethereum_value_from_log_data, referral_fee_for_token, referral_fee_from_transfers};

pub(crate) const FUNCTION_OKX_DAG_SWAP_BY_ORDER_ID: &str = "0xf2c42696";
pub(crate) const FUNCTION_OKX_UNISWAP_V3_SWAP_TO: &str = "0x0d5f0e3b";
pub(crate) const FUNCTION_OKX_UNXSWAP_BY_ORDER_ID: &str = "0x9871efa4";
pub struct OkxParser;

struct ReceiptTransfer {
    token: String,
    from: String,
    to: String,
    value: String,
}

impl TransactionParser<ParseContext<'_>, PrimitivesTransaction> for OkxParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        Self::matches_selector(&context.transaction.input)
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let referral_fee = Self::referral_fee_from_commissions(context).or_else(|| referral_fee_from_transfers(*context.metadata.chain, context.metadata.receipt));
        let metadata = Self::try_map_receipt_swap(context)?.with_referral_fee(referral_fee);

        context.make_swap_transaction(&context.transaction.from, &context.transaction.from, &metadata)
    }
}

impl OkxParser {
    fn matches_selector(input: &str) -> bool {
        input.starts_with(FUNCTION_OKX_DAG_SWAP_BY_ORDER_ID) || input.starts_with(FUNCTION_OKX_UNISWAP_V3_SWAP_TO) || input.starts_with(FUNCTION_OKX_UNXSWAP_BY_ORDER_ID)
    }

    fn try_map_receipt_swap(context: &ParseContext<'_>) -> Option<TransactionSwapMetadata> {
        Self::try_map_receipt_event(context).or_else(|| Self::try_map_transfer_swap(context))
    }

    fn try_map_receipt_event(context: &ParseContext<'_>) -> Option<TransactionSwapMetadata> {
        let event = context.metadata.receipt.logs.iter().find_map(Log::decode_event::<IOkxDexRouter::OrderRecord>)?;
        if event.sender != Address::from_str(&context.transaction.from).ok()? {
            return None;
        }

        let from_asset = Self::asset_id_from_token(context, &event.fromToken.to_checksum(None))?;
        let to_asset = Self::asset_id_from_token(context, &event.toToken.to_checksum(None))?;
        if from_asset == to_asset {
            return None;
        }

        Some(TransactionSwapMetadata::new(from_asset, u256_to_biguint(&event.fromAmount), to_asset, u256_to_biguint(&event.returnAmount), SwapProvider::Okx))
    }

    fn referral_fee_from_commissions(context: &ParseContext<'_>) -> Option<TransactionSwapReferralFee> {
        let referral = Address::from_str(EVM_REFERRAL_ADDRESS).ok()?;
        context.metadata.receipt.logs.iter().find_map(|log| {
            let commission = log
                .decode_event::<IOkxDexRouter::CommissionFromTokenRecord>()
                .map(OkxCommission::from)
                .or_else(|| log.decode_event::<IOkxDexRouter::CommissionToTokenRecord>().map(OkxCommission::from))?;
            (commission.referrer == referral).then(|| referral_fee_for_token(*context.metadata.chain, &commission.token.to_checksum(None), u256_to_biguint(&commission.amount)))?
        })
    }

    fn try_map_transfer_swap(context: &ParseContext<'_>) -> Option<TransactionSwapMetadata> {
        let from = ethereum_address_checksum(&context.transaction.from).ok()?;
        let transfers: Vec<ReceiptTransfer> = context.metadata.receipt.logs.iter().filter_map(ReceiptTransfer::from_log).collect();
        let outgoing: Vec<&ReceiptTransfer> = transfers.iter().filter(|transfer| transfer.from == from && transfer.value != "0").collect();
        let incoming: Vec<&ReceiptTransfer> = transfers.iter().filter(|transfer| transfer.to == from && transfer.value != "0").collect();

        match (context.transaction.value > BigUint::from(0u8), outgoing.as_slice(), incoming.as_slice()) {
            (_, [sent], [received]) if sent.token != received.token => Some(TransactionSwapMetadata::new(
                AssetId::from_token(*context.metadata.chain, &sent.token),
                BigUint::from_str(&sent.value).ok()?,
                AssetId::from_token(*context.metadata.chain, &received.token),
                BigUint::from_str(&received.value).ok()?,
                SwapProvider::Okx,
            )),
            (true, [], [received]) => Some(TransactionSwapMetadata::new(
                AssetId::from_chain(*context.metadata.chain),
                context.transaction.value.clone(),
                AssetId::from_token(*context.metadata.chain, &received.token),
                BigUint::from_str(&received.value).ok()?,
                SwapProvider::Okx,
            )),
            _ => None,
        }
    }

    fn asset_id_from_token(context: &ParseContext<'_>, token: &str) -> Option<AssetId> {
        let token = ethereum_address_checksum(token).ok()?;
        if token == EVM_NATIVE_TOKEN_ADDRESS || token == Address::ZERO.to_checksum(None) {
            return Some(AssetId::from_chain(*context.metadata.chain));
        }

        Some(AssetId::from_token(*context.metadata.chain, &token))
    }
}

impl ReceiptTransfer {
    fn from_log(log: &Log) -> Option<Self> {
        if log.topics.len() != 3 || log.topics.first().is_none_or(|topic| topic != TRANSFER_TOPIC) {
            return None;
        }

        Some(Self {
            token: ethereum_address_checksum(&log.address).ok()?,
            from: ethereum_address_from_topic(log.topics.get(1)?)?,
            to: ethereum_address_from_topic(log.topics.get(2)?)?,
            value: ethereum_value_from_log_data(&log.data, 0, EVENT_WORD_SIZE)?.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ethereum_address_checksum;
    use crate::rpc::model::{Log, Transaction, TransactionReceipt};
    use crate::rpc::parsers::ProtocolParsers;
    use alloy_sol_types::SolEvent;
    use chrono::DateTime;
    use num_bigint::BigUint;
    use primitives::{Chain, SwapProvider, TransactionState, asset_constants::BASE_USDC_TOKEN_ID, testkit::json_rpc::load_json_rpc_result};

    fn map_transaction(chain: &Chain, transaction: &Transaction, receipt: &TransactionReceipt) -> PrimitivesTransaction {
        ProtocolParsers::map_transaction(chain, transaction, receipt, DateTime::from_timestamp(1743373403, 0).unwrap()).unwrap()
    }

    #[test]
    fn test_map_okx_transactions() {
        let receipt_tx = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/okx_base_swap_tx.json"));
        let receipt_only = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/okx_base_swap_tx_receipt.json"));
        let swap_tx = map_transaction(&Chain::Base, &receipt_tx, &receipt_only);
        let swap_metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.clone().unwrap()).unwrap();
        assert_eq!(swap_tx.transaction_type, primitives::TransactionType::Swap);
        assert_eq!(swap_tx.state, TransactionState::Confirmed);
        assert_eq!(swap_metadata.provider, Some(SwapProvider::Okx.id().to_string()));
        assert_eq!(
            swap_metadata.from_asset,
            AssetId {
                chain: Chain::Base,
                token_id: Some(BASE_USDC_TOKEN_ID.to_string()),
            }
        );
        assert_eq!(
            swap_metadata.to_asset,
            AssetId {
                chain: Chain::Base,
                token_id: Some("0x0000000f2eB9f69274678c76222B35eEc7588a65".to_string()),
            }
        );
        assert_eq!(swap_metadata.from_value, BigUint::from(995000u64));
        assert_eq!(swap_metadata.to_value, BigUint::from(928345u64));

        let transfer_tx = Transaction {
            from: "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F".to_string(),
            gas: 750000,
            hash: "0x77144af6766c014ad05b0ae90979dc5df9978ecb5829c89925659445b8630dd2".to_string(),
            input: FUNCTION_OKX_DAG_SWAP_BY_ORDER_ID.to_string(),
            to: Some("0x4409921ae43a39a11d90f7b7f96cfd0b8093d9fc".to_string()),
            value: BigUint::from(0u8),
            calls: None,
        };
        let transfer_receipt = TransactionReceipt {
            gas_used: BigUint::from(318420u32),
            effective_gas_price: BigUint::from(10_000_000u64),
            logs: vec![
                Log::mock_erc20_transfer(BASE_USDC_TOKEN_ID, "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F", "0x4409921ae43a39a11d90f7b7f96cfd0b8093d9fc", 995000),
                Log::mock_erc20_transfer("0x0000000f2eB9f69274678c76222B35eEc7588a65", "0x4409921ae43a39a11d90f7b7f96cfd0b8093d9fc", "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F", 928345),
            ],
            block_number: 1,
            ..TransactionReceipt::mock()
        };
        let transfer_swap_tx = map_transaction(&Chain::Base, &transfer_tx, &transfer_receipt);
        let transfer_metadata: TransactionSwapMetadata = serde_json::from_value(transfer_swap_tx.metadata.clone().unwrap()).unwrap();
        assert_eq!(transfer_swap_tx.transaction_type, primitives::TransactionType::Swap);
        assert_eq!(transfer_metadata.provider, Some(SwapProvider::Okx.id().to_string()));
        assert_eq!(
            transfer_metadata.from_asset,
            AssetId {
                chain: Chain::Base,
                token_id: Some(BASE_USDC_TOKEN_ID.to_string()),
            }
        );
        assert_eq!(
            transfer_metadata.to_asset,
            AssetId {
                chain: Chain::Base,
                token_id: Some("0x0000000f2eB9f69274678c76222B35eEc7588a65".to_string()),
            }
        );
        assert_eq!(transfer_metadata.from_value, BigUint::from(995000u64));
        assert_eq!(transfer_metadata.to_value, BigUint::from(928345u64));

        let uniswap_v3_swap_to_tx = Transaction {
            from: "0xAdaf6f9B702718E3CEC12F944be7dF8b34E59E2f".to_string(),
            gas: 920000,
            hash: "0x336524846fec8f7a4a37ebac417f3ddd2d25b6fdf9b9cf0b88e1f69bb5601393".to_string(),
            input: "0x0d5f0e3b00000000000000003bbc864aadaf6f9b702718e3cec12f944be7df8b34e59e2f00000000000000000000000000000000000000000000043c33c19375648000000000000000000000000000000000000000000000000000000036e5945adfeb74000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000012000000000000000000000009bdc7dfd19b75b023e28bbb8e197295c51ce55e4777777771111800000000000000000000000000000000000003798ea0b0a14fd777777771111000000000064fa00a9ed787f3793db668bff3e6e6e7db0f92a1b800000000000000000000000eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee3ca20afc2bbb0000004c4b400d9dab1a248f63b0a48965ba8435e4de7497a3dc".to_string(),
            to: Some("0x5e1f62dac767b0491e3ce72469c217365d5b48cc".to_string()),
            value: BigUint::from(0u8),
            calls: None,
        };
        let uniswap_v3_swap_to_receipt = TransactionReceipt {
            gas_used: BigUint::from(203405u32),
            effective_gas_price: BigUint::from(230068341u32),
            logs: vec![Log {
                address: "0x5e1f62dac767b0491e3ce72469c217365d5b48cc".to_string(),
                topics: vec![IOkxDexRouter::OrderRecord::SIGNATURE_HASH.to_string()],
                data: "0x00000000000000000000000052498f8d9791736f1d6398fe95ba3bd868114d10000000000000000000000000eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee000000000000000000000000adaf6f9b702718e3cec12f944be7df8b34e59e2f00000000000000000000000000000000000000000000043c33c1937564800000000000000000000000000000000000000000000000000000003798ea0b0a14fd".to_string(),
                transaction_hash: None,
            }],
            block_number: 24717134,
            ..TransactionReceipt::mock()
        };
        let uniswap_v3_swap_to = map_transaction(&Chain::Ethereum, &uniswap_v3_swap_to_tx, &uniswap_v3_swap_to_receipt);
        let uniswap_v3_swap_to_metadata: TransactionSwapMetadata = serde_json::from_value(uniswap_v3_swap_to.metadata.clone().unwrap()).unwrap();
        assert_eq!(uniswap_v3_swap_to.transaction_type, primitives::TransactionType::Swap);
        assert_eq!(uniswap_v3_swap_to_metadata.provider, Some(SwapProvider::Okx.id().to_string()));
        assert_eq!(
            uniswap_v3_swap_to_metadata.from_asset,
            AssetId::from_token(Chain::Ethereum, &ethereum_address_checksum("0x52498f8d9791736f1d6398fe95ba3bd868114d10").unwrap())
        );
        assert_eq!(uniswap_v3_swap_to_metadata.to_asset, AssetId { chain: Chain::Ethereum, token_id: None });
        assert_eq!(uniswap_v3_swap_to_metadata.from_value, BigUint::parse_bytes(b"20000000000000000000000", 10).unwrap());
        assert_eq!(uniswap_v3_swap_to_metadata.to_value, BigUint::from(15649254694065405u64));

        let unxswap_by_order_id_tx = Transaction {
            from: "0xAdaf6f9B702718E3CEC12F944be7dF8b34E59E2f".to_string(),
            gas: 920000,
            hash: "0xf3714f46b23016a349e549e6212c6e39fd3f2ef3926039775377ba70d962bfa5".to_string(),
            input: "0x9871efa400000000000000003bbc864a249e38ea4102d0cf8264d3701f1a0e39c4f2dc3b000000000000000000000000000000000000000001c47e5d3263f59c9d062a020000000000000000000000000000000000000000000000000020068f78c840c70000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000170000000000000003b6d034097e1fcb93ae7267dbafad23f7b9afaa08264cfd87777777711118000000000000000000000000000000000000020595fca29f3dc777777771111000000000064fa00a9ed787f3793db668bff3e6e6e7db0f92a1b800000000000000000000000eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee3ca20afc2bbb0000004c4b400d9dab1a248f63b0a48965ba8435e4de7497a3dc".to_string(),
            to: Some("0x5e1f62dac767b0491e3ce72469c217365d5b48cc".to_string()),
            value: BigUint::from(0u8),
            calls: None,
        };
        let unxswap_by_order_id_receipt = TransactionReceipt {
            gas_used: BigUint::from(176410u32),
            effective_gas_price: BigUint::from(221977999u32),
            logs: vec![Log {
                address: "0x5e1f62dac767b0491e3ce72469c217365d5b48cc".to_string(),
                topics: vec![IOkxDexRouter::OrderRecord::SIGNATURE_HASH.to_string()],
                data: "0x000000000000000000000000249e38ea4102d0cf8264d3701f1a0e39c4f2dc3b000000000000000000000000eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee000000000000000000000000adaf6f9b702718e3cec12f944be7df8b34e59e2f000000000000000000000000000000000000000001c47e5d3263f59c9d062a020000000000000000000000000000000000000000000000000020595fca29f3dc".to_string(),
                transaction_hash: None,
            }],
            block_number: 24717121,
            ..TransactionReceipt::mock()
        };
        let unxswap_by_order_id = map_transaction(&Chain::Ethereum, &unxswap_by_order_id_tx, &unxswap_by_order_id_receipt);
        let unxswap_by_order_id_metadata: TransactionSwapMetadata = serde_json::from_value(unxswap_by_order_id.metadata.clone().unwrap()).unwrap();
        assert_eq!(unxswap_by_order_id.transaction_type, primitives::TransactionType::Swap);
        assert_eq!(unxswap_by_order_id_metadata.provider, Some(SwapProvider::Okx.id().to_string()));
        assert_eq!(
            unxswap_by_order_id_metadata.from_asset,
            AssetId::from_token(Chain::Ethereum, &ethereum_address_checksum("0x249e38ea4102d0cf8264d3701f1a0e39c4f2dc3b").unwrap())
        );
        assert_eq!(unxswap_by_order_id_metadata.to_asset, AssetId { chain: Chain::Ethereum, token_id: None });
        assert_eq!(unxswap_by_order_id_metadata.from_value, BigUint::parse_bytes(b"547031207820868594841299458", 10).unwrap());
        assert_eq!(unxswap_by_order_id_metadata.to_value, BigUint::from(9105467203253212u64));

        let mut reverted_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/okx_base_swap_tx_receipt.json"));
        reverted_receipt.status = "0x0".to_string();
        let reverted_tx = map_transaction(&Chain::Base, &receipt_tx, &reverted_receipt);
        assert_eq!(reverted_tx.transaction_type, primitives::TransactionType::Swap);
        assert_eq!(reverted_tx.state, TransactionState::Reverted);
    }
}
