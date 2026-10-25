use primitives::{SwapProvider, Transaction as PrimitivesTransaction};

use super::{ParseContext, ParseContextExt, TransactionParser, referral_fee_from_transfers, transfer_swap};
use crate::uniswap::deployment::aero::METAROUTER;

pub struct AeroParser;

impl TransactionParser<ParseContext<'_>, PrimitivesTransaction> for AeroParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        context.is_to(METAROUTER)
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let metadata = transfer_swap(context, SwapProvider::Aero)?.with_referral_fee(referral_fee_from_transfers(*context.metadata.chain, context.metadata.receipt));
        context.make_swap_transaction(&context.transaction.from, &context.transaction.from, &metadata)
    }
}

#[cfg(test)]
mod tests {
    use crate::rpc::{
        model::{Log, Transaction, TransactionReceipt},
        parsers::ProtocolParsers,
    };
    use crate::uniswap::deployment::aero::METAROUTER;
    use chrono::DateTime;
    use num_bigint::BigUint;
    use primitives::{
        AssetId, Chain, SwapProvider, TransactionSwapMetadata, TransactionSwapReferralFee,
        asset_constants::{ARC_EURC_TOKEN_ID, ARC_USDC_TOKEN_ID, BASE_USDC_ASSET_ID},
        swap::EVM_REFERRAL_ADDRESS,
        testkit::json_rpc::load_json_rpc_result,
    };

    const TEST_WALLET: &str = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
    const TEST_POOL: &str = "0x784e539f15ca2d19449a6311c5dab6cf37d8f1ba";

    fn map_swap(chain: Chain, transaction: &Transaction, receipt: &TransactionReceipt) -> Option<TransactionSwapMetadata> {
        ProtocolParsers::map_transaction(&chain, transaction, receipt, DateTime::default())?.swap_metadata()
    }

    #[test]
    fn test_map_aero_swap() {
        let eth_usdc = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/aero_eth_usdc_tx.json"));
        let eth_usdc_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/aero_eth_usdc_tx_receipt.json"));
        let usdc_eth = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/aero_usdc_eth_tx.json"));
        let usdc_eth_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/aero_usdc_eth_tx_receipt.json"));
        let wallet_swap = Transaction {
            from: TEST_WALLET.to_string(),
            to: Some(METAROUTER.to_string()),
            value: BigUint::ZERO,
            ..usdc_eth.clone()
        };

        assert_eq!(
            map_swap(Chain::Base, &eth_usdc, &eth_usdc_receipt),
            Some(TransactionSwapMetadata::new(
                AssetId::from_chain(Chain::Base),
                BigUint::from(250_000_000_000_000_000u64),
                BASE_USDC_ASSET_ID.clone(),
                BigUint::from(586_941_708u64),
                SwapProvider::Aero
            ))
        );
        assert_eq!(
            map_swap(Chain::Base, &usdc_eth, &usdc_eth_receipt),
            Some(TransactionSwapMetadata::new(
                BASE_USDC_ASSET_ID.clone(),
                BigUint::from(6_089_940u64),
                AssetId::from_chain(Chain::Base),
                BigUint::from(2_592_836_864_422_313u64),
                SwapProvider::Aero
            ))
        );
        assert_eq!(
            map_swap(
                Chain::Arc,
                &wallet_swap,
                &TransactionReceipt {
                    logs: vec![
                        Log::mock_erc20_transfer(ARC_EURC_TOKEN_ID, TEST_WALLET, TEST_POOL, 10_000_000),
                        Log::mock_erc20_transfer(ARC_USDC_TOKEN_ID, METAROUTER, EVM_REFERRAL_ADDRESS, 55_000),
                        Log::mock_erc20_transfer(ARC_USDC_TOKEN_ID, METAROUTER, TEST_WALLET, 11_000_000),
                    ],
                    ..usdc_eth_receipt.clone()
                }
            ),
            Some(
                TransactionSwapMetadata::new(
                    AssetId::from_token(Chain::Arc, ARC_EURC_TOKEN_ID),
                    BigUint::from(10_000_000u64),
                    AssetId::from_chain(Chain::Arc),
                    BigUint::from(11_000_000_000_000_000_000u128),
                    SwapProvider::Aero
                )
                .with_referral_fee(Some(TransactionSwapReferralFee {
                    asset_id: AssetId::from_chain(Chain::Arc),
                    value: BigUint::from(55_000_000_000_000_000u64)
                }))
            )
        );
    }
}
