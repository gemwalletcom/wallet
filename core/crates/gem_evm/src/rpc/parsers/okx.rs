use alloy_primitives::{Address, B256};
use alloy_sol_types::SolInterface;
use num_bigint::BigUint;
use std::str::FromStr;

use crate::{
    contracts::{
        IOkxDexRouter::{self, BaseRequest, IOkxDexRouterCalls},
        OkxCommission,
    },
    ethereum_address_checksum,
    rpc::model::Log,
    u256::u256_to_biguint,
};
use primitives::{AssetId, SwapProvider, Transaction as PrimitivesTransaction, TransactionState, TransactionSwapMetadata, TransactionSwapReferralFee, contract_constants::EVM_NATIVE_TOKEN_ADDRESS, decode_hex, swap::EVM_REFERRAL_ADDRESS};

use super::{ParseContext, ParseContextExt, TransactionParser, referral_fee_for_token, referral_fee_from_transfers, transfer_swap};

const SELECTOR_HEX_LENGTH: usize = 10;

pub struct OkxParser;

impl TransactionParser<ParseContext<'_>, PrimitivesTransaction> for OkxParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        Self::matches_selector(&context.transaction.input)
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let referral_fee = Self::referral_fee_from_commissions(context).or_else(|| referral_fee_from_transfers(*context.metadata.chain, context.metadata.receipt));
        let metadata = Self::try_map_receipt_swap(context).or_else(|| Self::try_map_reverted_request(context))?.with_referral_fee(referral_fee);

        context.make_swap_transaction(&context.transaction.from, &context.transaction.from, &metadata)
    }
}

impl OkxParser {
    fn matches_selector(input: &str) -> bool {
        input
            .get(..SELECTOR_HEX_LENGTH)
            .and_then(|selector| decode_hex(selector).ok())
            .and_then(|selector| selector.try_into().ok())
            .is_some_and(IOkxDexRouterCalls::valid_selector)
    }

    fn try_map_reverted_request(context: &ParseContext<'_>) -> Option<TransactionSwapMetadata> {
        if context.metadata.receipt.get_state() != TransactionState::Reverted {
            return None;
        }
        let request = Self::base_request(&decode_hex(&context.transaction.input).ok()?)?;
        let from_asset = Self::asset_id_from_token(context, &Address::from_word(B256::from(request.fromToken)).to_checksum(None))?;
        let to_asset = Self::asset_id_from_token(context, &request.toToken.to_checksum(None))?;
        let from_value = match from_asset.is_native() {
            true => context.transaction.value.clone(),
            false => u256_to_biguint(&request.fromTokenAmount),
        };
        Some(TransactionSwapMetadata::new(from_asset, from_value, to_asset, u256_to_biguint(&request.minReturnAmount), SwapProvider::Okx))
    }

    fn base_request(input: &[u8]) -> Option<BaseRequest> {
        match IOkxDexRouterCalls::abi_decode(input).ok()? {
            IOkxDexRouterCalls::smartSwapByOrderId(call) => Some(call.baseRequest),
            IOkxDexRouterCalls::smartSwapTo(call) => Some(call.baseRequest),
            IOkxDexRouterCalls::dagSwapByOrderId(call) => Some(call.baseRequest),
            IOkxDexRouterCalls::dagSwapTo(call) => Some(call.baseRequest),
            IOkxDexRouterCalls::swapWrapToWithBaseRequest(call) => Some(call.baseRequest),
            IOkxDexRouterCalls::uniswapV3SwapToWithBaseRequest(call) => Some(call.baseRequest),
            IOkxDexRouterCalls::unxswapToWithBaseRequest(call) => Some(call.baseRequest),
            IOkxDexRouterCalls::uniswapV3SwapTo(_) | IOkxDexRouterCalls::unxswapByOrderId(_) | IOkxDexRouterCalls::unxswapTo(_) => None,
        }
    }

    fn try_map_receipt_swap(context: &ParseContext<'_>) -> Option<TransactionSwapMetadata> {
        Self::try_map_receipt_event(context).or_else(|| transfer_swap(context, SwapProvider::Okx, &Self::native_commission(context)))
    }

    fn native_commission(context: &ParseContext<'_>) -> BigUint {
        let native = AssetId::from_chain(*context.metadata.chain);
        context
            .metadata
            .receipt
            .logs
            .iter()
            .filter_map(Log::decode_event::<IOkxDexRouter::CommissionToTokenRecord>)
            .filter(|event| Self::asset_id_from_token(context, &event.toTokenAddress.to_checksum(None)).as_ref() == Some(&native))
            .map(|event| u256_to_biguint(&event.commissionAmount))
            .sum()
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

        let from_value = u256_to_biguint(&event.fromAmount) + Self::input_commission(context, event.fromToken);
        Some(TransactionSwapMetadata::new(from_asset, from_value, to_asset, u256_to_biguint(&event.returnAmount), SwapProvider::Okx))
    }

    fn input_commission(context: &ParseContext<'_>, from_token: Address) -> BigUint {
        context
            .metadata
            .receipt
            .logs
            .iter()
            .filter_map(Log::decode_event::<IOkxDexRouter::CommissionFromTokenRecord>)
            .filter(|event| event.fromTokenAddress == from_token)
            .map(|event| u256_to_biguint(&event.commissionAmount))
            .sum()
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

    fn asset_id_from_token(context: &ParseContext<'_>, token: &str) -> Option<AssetId> {
        let token = ethereum_address_checksum(token).ok()?;
        if token == EVM_NATIVE_TOKEN_ADDRESS || token == Address::ZERO.to_checksum(None) {
            return Some(AssetId::from_chain(*context.metadata.chain));
        }

        Some(AssetId::from_token(*context.metadata.chain, &token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ethereum_address_checksum;
    use crate::rpc::model::{Log, Transaction, TransactionReceipt};
    use crate::rpc::parsers::{ProtocolParsers, WITHDRAWAL_TOPIC};
    use alloy_primitives::U256;
    use alloy_sol_types::{SolCall, SolEvent};
    use chrono::DateTime;
    use num_bigint::BigUint;
    use primitives::{
        Chain, SwapProvider, TransactionState, TransactionType,
        asset_constants::{BASE_USDC_TOKEN_ID, BASE_WETH_TOKEN_ID, ETHEREUM_USDC_TOKEN_ID},
        hex::encode_with_0x,
        testkit::json_rpc::load_json_rpc_result,
    };

    const COMMISSION_TRAILER: &str = "800000000000000000000000eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee3ca20afc2aaa0000004c4b400d9dab1a248f63b0a48965ba8435e4de7497a3dc";

    fn map_transaction(chain: &Chain, transaction: &Transaction, receipt: &TransactionReceipt) -> PrimitivesTransaction {
        ProtocolParsers::map_transaction(chain, transaction, receipt, DateTime::from_timestamp(1743373403, 0).unwrap()).unwrap()
    }

    #[test]
    fn test_map_okx_transactions() {
        let receipt_tx = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/okx_base_swap_tx.json"));
        let receipt_only = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/okx_base_swap_tx_receipt.json"));
        let swap_tx = map_transaction(&Chain::Base, &receipt_tx, &receipt_only);
        let swap_metadata = swap_tx.swap_metadata().unwrap();
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

        let commission_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/okx_base_swap_commission_tx_receipt.json"));
        let commission_metadata = map_transaction(&Chain::Base, &receipt_tx, &commission_receipt).swap_metadata().unwrap();
        assert_eq!(commission_metadata.from_value, BigUint::from(1_000_000u64));
        assert_eq!(commission_metadata.to_value, BigUint::from(928345u64));
        assert_eq!(
            commission_metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: AssetId::from_token(Chain::Base, BASE_USDC_TOKEN_ID),
                value: BigUint::from(5000u64)
            })
        );

        let transfer_tx = Transaction {
            from: "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F".to_string(),
            gas: 750000,
            hash: "0x77144af6766c014ad05b0ae90979dc5df9978ecb5829c89925659445b8630dd2".to_string(),
            input: encode_with_0x(&IOkxDexRouter::dagSwapByOrderIdCall::SELECTOR),
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
        let transfer_metadata = transfer_swap_tx.swap_metadata().unwrap();
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
        let uniswap_v3_swap_to_metadata = uniswap_v3_swap_to.swap_metadata().unwrap();
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
        let unxswap_by_order_id_metadata = unxswap_by_order_id.swap_metadata().unwrap();
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

        let unwrap_receipt = TransactionReceipt {
            logs: vec![
                Log::mock_erc20_transfer(BASE_USDC_TOKEN_ID, &transfer_tx.from, transfer_tx.to.as_deref().unwrap(), 10_000_000),
                Log {
                    address: BASE_WETH_TOKEN_ID.to_string(),
                    topics: vec![WITHDRAWAL_TOPIC.to_string(), format!("0x{:0>64}", "dead")],
                    data: format!("0x{:064x}", 5_000_000_000_000_000u64),
                    transaction_hash: None,
                },
                Log {
                    address: BASE_WETH_TOKEN_ID.to_string(),
                    topics: vec![WITHDRAWAL_TOPIC.to_string(), format!("0x{:0>64}", &transfer_tx.to.as_deref().unwrap()[2..])],
                    data: format!("0x{:064x}", 1_000_000_000_000_000u64),
                    transaction_hash: None,
                },
                Log {
                    address: transfer_tx.to.clone().unwrap(),
                    topics: vec![IOkxDexRouter::CommissionToTokenRecord::SIGNATURE_HASH.to_string()],
                    data: encode_with_0x(
                        &IOkxDexRouter::CommissionToTokenRecord {
                            toTokenAddress: EVM_NATIVE_TOKEN_ADDRESS.parse().unwrap(),
                            commissionAmount: U256::from(7_000_000_000_000u64),
                            referrerAddress: EVM_REFERRAL_ADDRESS.parse().unwrap(),
                            commissionRate: U256::from(70),
                        }
                        .encode_data(),
                    ),
                    transaction_hash: None,
                },
            ],
            ..transfer_receipt
        };
        assert_eq!(
            map_transaction(&Chain::Base, &transfer_tx, &unwrap_receipt).swap_metadata(),
            Some(
                TransactionSwapMetadata::new(
                    AssetId::from_token(Chain::Base, BASE_USDC_TOKEN_ID),
                    BigUint::from(10_000_000u64),
                    AssetId::from_chain(Chain::Base),
                    BigUint::from(993_000_000_000_000u64),
                    SwapProvider::Okx
                )
                .with_referral_fee(Some(TransactionSwapReferralFee {
                    asset_id: AssetId::from_chain(Chain::Base),
                    value: BigUint::from(7_000_000_000_000u64)
                }))
            )
        );
    }

    #[test]
    fn test_map_okx_reverted_request() {
        let usdc = Address::from_str(ETHEREUM_USDC_TOKEN_ID).unwrap();
        let native = Address::from_str(EVM_NATIVE_TOKEN_ADDRESS).unwrap();
        let reverted = TransactionReceipt {
            status: "0x0".to_string(),
            ..TransactionReceipt::mock()
        };
        let token_to_native = BaseRequest {
            fromToken: U256::from_be_bytes(usdc.into_word().0),
            toToken: native,
            fromTokenAmount: U256::from(995_000u64),
            minReturnAmount: U256::from(400_000_000_000_000u64),
            deadLine: U256::from(1_800_000_000u64),
        };
        let dag_swap = Transaction {
            to: Some("0x4409921ae43a39a11d90f7b7f96cfd0b8093d9fc".to_string()),
            input: format!(
                "{}{COMMISSION_TRAILER}",
                encode_with_0x(
                    &IOkxDexRouter::dagSwapByOrderIdCall {
                        orderId: U256::from(1),
                        baseRequest: token_to_native,
                        paths: vec![],
                    }
                    .abi_encode()
                )
            ),
            ..Transaction::mock()
        };
        let dag_swap_tx = map_transaction(&Chain::Ethereum, &dag_swap, &reverted);
        assert_eq!(dag_swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(dag_swap_tx.state, TransactionState::Reverted);
        assert_eq!(
            dag_swap_tx.swap_metadata(),
            Some(TransactionSwapMetadata::new(
                AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID),
                BigUint::from(995_000u64),
                AssetId::from_chain(Chain::Ethereum),
                BigUint::from(400_000_000_000_000u64),
                SwapProvider::Okx
            ))
        );
        assert_eq!(ProtocolParsers::map_transaction(&Chain::Ethereum, &dag_swap, &TransactionReceipt::mock(), DateTime::default()), None);

        let native_to_token = BaseRequest {
            fromToken: U256::from_be_bytes(native.into_word().0) | (U256::from(1) << 255),
            toToken: usdc,
            fromTokenAmount: U256::from(1_000_000_000_000_000_000u64),
            minReturnAmount: U256::from(2_400_000_000u64),
            deadLine: U256::from(1_800_000_000u64),
        };
        let smart_swap = Transaction {
            to: dag_swap.to.clone(),
            input: encode_with_0x(
                &IOkxDexRouter::smartSwapToCall {
                    orderId: U256::from(2),
                    receiver: Address::from_str(&dag_swap.from).unwrap(),
                    baseRequest: native_to_token,
                    batchesAmount: vec![],
                    batches: vec![],
                    pmmRequests: vec![],
                }
                .abi_encode(),
            ),
            value: BigUint::from(1_005_025_125_628_140_703u64),
            ..Transaction::mock()
        };
        assert_eq!(
            map_transaction(&Chain::Ethereum, &smart_swap, &reverted).swap_metadata(),
            Some(TransactionSwapMetadata::new(
                AssetId::from_chain(Chain::Ethereum),
                BigUint::from(1_005_025_125_628_140_703u64),
                AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID),
                BigUint::from(2_400_000_000u64),
                SwapProvider::Okx
            ))
        );

        let pools_only = Transaction {
            to: dag_swap.to.clone(),
            input: encode_with_0x(
                &IOkxDexRouter::uniswapV3SwapToCall {
                    receiver: U256::from_be_bytes(Address::from_str(&dag_swap.from).unwrap().into_word().0),
                    amount: U256::from(995_000u64),
                    minReturn: U256::from(400_000_000_000_000u64),
                    pools: vec![U256::from(1)],
                }
                .abi_encode(),
            ),
            ..Transaction::mock()
        };
        assert_eq!(ProtocolParsers::map_transaction(&Chain::Ethereum, &pools_only, &reverted, DateTime::default()), None);
    }
}
