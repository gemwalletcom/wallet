use alloy_primitives::{Address, B256};
use alloy_sol_types::{SolEvent, SolInterface, sol};

use primitives::{AssetId, SwapProvider, Transaction as PrimitivesTransaction, TransactionSwapMetadata, contract_constants::MAYAN_SWIFT_CONTRACT, decode_hex};

use self::{MayanFulfillHelper::MayanFulfillHelperCalls, MayanSwift::MayanSwiftCalls};
use super::{ParseContext, ProtocolParser};

sol! {
    interface MayanFulfillHelper {
        #[derive(Default)]
        struct PermitParams {
            uint256 value;
            uint256 deadline;
            uint8 v;
            bytes32 r;
            bytes32 s;
        }

        function directFulfill(
            address tokenIn,
            uint256 amountIn,
            address mayanProtocol,
            bytes mayanData,
            PermitParams permitParams
        ) external payable;
        function fulfillWithERC20(
            address tokenIn,
            uint256 amountIn,
            address fulfillToken,
            address swapProtocol,
            bytes swapData,
            address mayanProtocol,
            bytes mayanData,
            PermitParams permitParams
        ) external payable;
        function fulfillWithEth(
            uint256 amountIn,
            address fulfillToken,
            address swapProtocol,
            bytes swapData,
            address mayanProtocol,
            bytes mayanData
        ) external payable;
    }

    interface MayanSwift {
        #[derive(Default)]
        struct OrderParams {
            bytes32 trader;
            bytes32 tokenOut;
            uint64 minAmountOut;
            uint64 gasDrop;
            uint64 cancelFee;
            uint64 refundFee;
            uint64 deadline;
            bytes32 destAddr;
            uint16 destChainId;
            bytes32 referrerAddr;
            uint8 referrerBps;
            uint8 auctionMode;
            bytes32 random;
        }

        function fulfillSimple(
            uint256 fulfillAmount,
            bytes32 orderHash,
            uint16 srcChainId,
            bytes32 tokenIn,
            uint8 protocolBps,
            OrderParams params,
            bytes32 recepient,
            bool batch
        ) external payable returns (uint64 sequence);
        function fulfillOrder(uint256 fulfillAmount, bytes encodedVm, bytes32 recepient, bool batch) external payable returns (uint64 sequence);

        event OrderFulfilled(bytes32 key, uint64 sequence, uint256 netAmount);
    }
}

pub struct MayanParser;

impl ProtocolParser for MayanParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        decode_hex(&context.transaction.input).is_ok_and(|input| MayanFulfillHelperCalls::abi_decode(&input).is_ok())
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let input = decode_hex(&context.transaction.input).ok()?;
        let (input_token, amount, mayan_protocol, mayan_data) = match MayanFulfillHelperCalls::abi_decode(&input).ok()? {
            MayanFulfillHelperCalls::directFulfill(call) => (call.tokenIn, call.amountIn, call.mayanProtocol, call.mayanData),
            MayanFulfillHelperCalls::fulfillWithERC20(call) => (call.tokenIn, call.amountIn, call.mayanProtocol, call.mayanData),
            MayanFulfillHelperCalls::fulfillWithEth(call) => (Address::ZERO, call.amountIn, call.mayanProtocol, call.mayanData),
        };
        if amount.is_zero() || !mayan_protocol.to_checksum(None).eq_ignore_ascii_case(MAYAN_SWIFT_CONTRACT) {
            return None;
        }

        let (recipient, output_token, order_hash) = match MayanSwiftCalls::abi_decode(&mayan_data).ok()? {
            MayanSwiftCalls::fulfillSimple(call) => (Address::from_word(call.params.destAddr), Address::from_word(call.params.tokenOut), call.orderHash),
            MayanSwiftCalls::fulfillOrder(call) => Self::decode_fulfill_order(&call.encodedVm)?,
        };
        let output_amount = context.receipt.logs.iter().find_map(|log| {
            if !log.address.eq_ignore_ascii_case(MAYAN_SWIFT_CONTRACT)
                || log.topics.len() != 1
                || log.topics.first()?.parse::<B256>().ok()? != MayanSwift::OrderFulfilled::SIGNATURE_HASH
            {
                return None;
            }
            let (key, _, net_amount) = MayanSwift::OrderFulfilled::abi_decode_data(&decode_hex(&log.data).ok()?).ok()?;
            (order_hash == key).then_some(net_amount)
        })?;

        let metadata = TransactionSwapMetadata {
            from_asset: AssetId::from(*context.chain, (!input_token.is_zero()).then(|| input_token.to_checksum(None))),
            from_value: amount.to_string(),
            to_asset: AssetId::from(*context.chain, (!output_token.is_zero()).then(|| output_token.to_checksum(None))),
            to_value: output_amount.to_string(),
            provider: Some(SwapProvider::Mayan.id().to_string()),
        };

        context.make_swap_transaction(&context.transaction.from, &recipient.to_checksum(None), &metadata)
    }
}

impl MayanParser {
    fn decode_fulfill_order(encoded_vm: &[u8]) -> Option<(Address, Address, B256)> {
        const VAA_HEADER_SIZE: usize = 6;
        const VAA_SIGNATURE_SIZE: usize = 66;
        const VAA_BODY_HEADER_SIZE: usize = 51;
        const FULFILL_ACTION: u8 = 1;

        let signature_count = usize::from(*encoded_vm.get(5)?);
        let payload_offset = VAA_HEADER_SIZE
            .checked_add(signature_count.checked_mul(VAA_SIGNATURE_SIZE)?)?
            .checked_add(VAA_BODY_HEADER_SIZE)?;
        let payload = encoded_vm.get(payload_offset..)?;
        if payload.first() != Some(&FULFILL_ACTION) {
            return None;
        }

        let order_hash = B256::from_slice(payload.get(1..33)?);
        let recipient = B256::from_slice(payload.get(67..99)?);
        let output_token = B256::from_slice(payload.get(101..133)?);

        Some((Address::from_word(recipient), Address::from_word(output_token), order_hash))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy_primitives::{Address, B256, Bytes, U256};
    use alloy_sol_types::SolCall;
    use chrono::DateTime;
    use num_bigint::BigUint;

    use crate::rpc::{
        model::{Transaction, TransactionReceipt},
        parsers::ProtocolParsers,
    };
    use primitives::{
        AssetId, Chain, SwapProvider, TransactionSwapMetadata, TransactionType,
        asset_constants::{POLYGON_USDC_ASSET_ID, POLYGON_USDC_TOKEN_ID},
        hex,
        testkit::json_rpc::load_json_rpc_result,
    };

    use super::{MAYAN_SWIFT_CONTRACT, MayanFulfillHelper, MayanSwift};

    #[derive(Clone, Copy)]
    enum FulfillMethod {
        Direct,
        Erc20,
        Eth,
    }

    #[test]
    fn test_parse_mayan_fulfillments() {
        let cases = [
            (
                include_str!("../../../testdata/mayan_polygon_fulfillment_transaction.json"),
                include_str!("../../../testdata/mayan_polygon_fulfillment_receipt.json"),
                "0x2977b8919dF6A60E93089E0F4231A28899005302",
                "9906901",
                "43605973076395982882",
            ),
            (
                include_str!("../../../testdata/mayan_polygon_fulfillment_second_transaction.json"),
                include_str!("../../../testdata/mayan_polygon_fulfillment_second_receipt.json"),
                "0x466B037ace44C0134Dcebd965A4a22Aed6DEA027",
                "10395621",
                "54972588054242227627",
            ),
        ];

        for (transaction_json, receipt_json, from, from_value, to_value) in cases {
            let transaction = load_json_rpc_result::<Transaction>(transaction_json);
            let receipt = load_json_rpc_result::<TransactionReceipt>(receipt_json);
            let parsed = ProtocolParsers::map_transaction(&Chain::Polygon, &transaction, &receipt, DateTime::default()).unwrap();
            let metadata = serde_json::from_value::<TransactionSwapMetadata>(parsed.metadata.unwrap()).unwrap();

            assert_eq!(parsed.transaction_type, TransactionType::Swap);
            assert_eq!(parsed.from, from);
            assert_eq!(parsed.to, "0x2A49C84B7173e21f9116B2798735f87531526b36");
            assert_eq!(metadata.from_asset, POLYGON_USDC_ASSET_ID.clone());
            assert_eq!(metadata.from_value, from_value);
            assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Polygon));
            assert_eq!(metadata.to_value, to_value);
            assert_eq!(metadata.provider, Some(SwapProvider::Mayan.id().to_string()));
        }

        let order_hash = "0xe71714346ea207444f35893dd33f6fbcad9222ec8934ed02ae1722c983362fc6";
        let receipt = load_json_rpc_result::<TransactionReceipt>(cases[0].1);
        for (method, expected_from, expected_to) in [
            (FulfillMethod::Direct, POLYGON_USDC_ASSET_ID.clone(), POLYGON_USDC_ASSET_ID.clone()),
            (FulfillMethod::Erc20, POLYGON_USDC_ASSET_ID.clone(), AssetId::from_chain(Chain::Polygon)),
            (FulfillMethod::Eth, AssetId::from_chain(Chain::Polygon), POLYGON_USDC_ASSET_ID.clone()),
        ] {
            let transaction = fulfillment_transaction(cases[0].2, 9_906_901, order_hash, method);
            let parsed = ProtocolParsers::map_transaction(&Chain::Polygon, &transaction, &receipt, DateTime::default()).unwrap();
            let metadata = serde_json::from_value::<TransactionSwapMetadata>(parsed.metadata.unwrap()).unwrap();
            assert_eq!(metadata.from_asset, expected_from);
            assert_eq!(metadata.to_asset, expected_to);
        }

        let transaction = fulfillment_transaction(cases[0].2, 9_906_901, order_hash, FulfillMethod::Erc20);
        let mut mismatched_receipt = receipt;
        mismatched_receipt
            .logs
            .iter_mut()
            .find(|log| log.address.eq_ignore_ascii_case(MAYAN_SWIFT_CONTRACT) && log.topics.len() == 1)
            .unwrap()
            .data
            .replace_range(2..66, "0000000000000000000000000000000000000000000000000000000000000001");
        assert_eq!(
            ProtocolParsers::map_transaction(&Chain::Polygon, &transaction, &mismatched_receipt, DateTime::default()),
            None
        );
    }

    fn fulfillment_transaction(solver: &str, amount: u64, order_hash: &str, method: FulfillMethod) -> Transaction {
        let recipient = Address::from_str("0x2a49c84b7173e21f9116b2798735f87531526b36").unwrap().into_word();
        let token = Address::from_str(POLYGON_USDC_TOKEN_ID).unwrap();
        let output_token = match method {
            FulfillMethod::Direct | FulfillMethod::Eth => token.into_word(),
            FulfillMethod::Erc20 => B256::ZERO,
        };
        let mayan_data = MayanSwift::fulfillSimpleCall {
            fulfillAmount: U256::from(amount),
            orderHash: B256::from_str(order_hash).unwrap(),
            srcChainId: 1,
            tokenIn: B256::ZERO,
            protocolBps: 0,
            params: MayanSwift::OrderParams {
                tokenOut: output_token,
                destAddr: recipient,
                ..Default::default()
            },
            recepient: B256::ZERO,
            batch: false,
        }
        .abi_encode();
        let protocol = Address::from_str(MAYAN_SWIFT_CONTRACT).unwrap();
        let input = match method {
            FulfillMethod::Direct => MayanFulfillHelper::directFulfillCall {
                tokenIn: token,
                amountIn: U256::from(amount),
                mayanProtocol: protocol,
                mayanData: mayan_data.into(),
                permitParams: MayanFulfillHelper::PermitParams::default(),
            }
            .abi_encode(),
            FulfillMethod::Erc20 => MayanFulfillHelper::fulfillWithERC20Call {
                tokenIn: token,
                amountIn: U256::from(amount),
                fulfillToken: Address::ZERO,
                swapProtocol: Address::ZERO,
                swapData: Bytes::new(),
                mayanProtocol: protocol,
                mayanData: mayan_data.into(),
                permitParams: MayanFulfillHelper::PermitParams::default(),
            }
            .abi_encode(),
            FulfillMethod::Eth => MayanFulfillHelper::fulfillWithEthCall {
                amountIn: U256::from(amount),
                fulfillToken: token,
                swapProtocol: Address::ZERO,
                swapData: Bytes::new(),
                mayanProtocol: protocol,
                mayanData: mayan_data.into(),
            }
            .abi_encode(),
        };

        Transaction {
            from: solver.to_string(),
            gas: 1_000_000,
            hash: B256::ZERO.to_string(),
            input: hex::encode_with_0x(&input),
            to: None,
            value: BigUint::from(0u8),
            calls: None,
        }
    }
}
