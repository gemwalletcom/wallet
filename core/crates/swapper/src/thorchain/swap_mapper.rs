use num_bigint::BigUint;
use primitives::{
    AssetId, Chain, TransactionSwapMetadata, TransactionSwapReferralFee,
    swap::{SwapStatus, THORCHAIN_REFERRAL_ADDRESS},
};

use super::chain::ChainName;
use super::constants::RUNE_ASSET;
use super::memo::ThorchainMemo;
use super::model::{TransactionCoin, TransactionStatus};
use crate::{SwapResult, SwapperProvider, fees::bps_of};

pub fn map_swap_result(response: &TransactionStatus) -> SwapResult {
    let status = response.swap_status();
    let eta_in_seconds = response.eta_in_seconds();

    let Some(ref transaction) = response.tx else {
        return SwapResult { status, metadata: None, eta_in_seconds };
    };

    if ChainName::from_symbol(&transaction.chain).is_none() {
        return SwapResult { status, metadata: None, eta_in_seconds };
    }

    let from_coin = transaction.coins.first();
    let from_asset = from_coin.and_then(TransactionCoin::asset_id);
    let from_value = from_coin.and_then(TransactionCoin::native_value);

    let out_coin = response.destination_coin();
    let to_asset = out_coin.and_then(TransactionCoin::asset_id);
    let to_value = out_coin.and_then(TransactionCoin::native_value);

    let metadata = match (from_asset, from_value, to_asset, to_value) {
        (Some(from_asset), Some(from_value), Some(to_asset), Some(to_value)) => {
            let referral_fee = map_referral_fee(response, &transaction.memo, &from_asset, &from_value);
            Some(TransactionSwapMetadata::new(from_asset, from_value, to_asset, to_value, SwapperProvider::Thorchain).with_referral_fee(referral_fee))
        }
        _ => None,
    };

    SwapResult { status, metadata, eta_in_seconds }
}

fn map_referral_fee(response: &TransactionStatus, memo: &str, from_asset: &AssetId, from_value: &BigUint) -> Option<TransactionSwapReferralFee> {
    let memo = ThorchainMemo::parse(memo)?;
    if memo.affiliate.as_deref() != Some(THORCHAIN_REFERRAL_ADDRESS) {
        return None;
    }
    let payout = response
        .out_txs
        .iter()
        .flatten()
        .filter(|transaction| transaction.to_address.as_deref().is_some_and(|address| address != memo.address))
        .filter_map(|transaction| transaction.coins.first())
        .find(|coin| coin.asset == RUNE_ASSET);
    match payout {
        Some(coin) => Some(TransactionSwapReferralFee {
            asset_id: Chain::Thorchain.as_asset_id(),
            value: coin.amount.parse().ok()?,
        }),
        None if response.swap_status() == SwapStatus::Completed => Some(TransactionSwapReferralFee {
            asset_id: from_asset.clone(),
            value: bps_of(from_value, memo.affiliate_bps.filter(|bps| *bps > 0)?),
        }),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{
        SwapProvider,
        asset_constants::{ETHEREUM_USDT_ASSET_ID, THORCHAIN_TCY_ASSET_ID, TRON_USDT_ASSET_ID},
    };

    fn status(json: &str) -> TransactionStatus {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_map_swap_result_ltc_to_tron_usdt() {
        let response = status(include_str!("testdata/tx_status_ltc_to_tron_usdt.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(
                    TransactionSwapMetadata::new(Chain::Litecoin.as_asset_id(), BigUint::from(160661010u64), TRON_USDT_ASSET_ID.clone(), BigUint::from(79158429u64), SwapProvider::Thorchain).with_referral_fee(Some(
                        TransactionSwapReferralFee {
                            asset_id: Chain::Thorchain.as_asset_id(),
                            value: BigUint::from(107275600u64)
                        }
                    ))
                ),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_ltc_to_eth() {
        let response = status(include_str!("testdata/tx_status_ltc_to_eth.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(
                    TransactionSwapMetadata::new(
                        Chain::Litecoin.as_asset_id(),
                        BigUint::from(5000000u64),
                        Chain::Ethereum.as_asset_id(),
                        BigUint::from(1243680000000000u64),
                        SwapProvider::Thorchain
                    )
                    .with_referral_fee(Some(TransactionSwapReferralFee {
                        asset_id: Chain::Thorchain.as_asset_id(),
                        value: BigUint::from(3352043u64)
                    }))
                ),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_btc_to_tron_pending() {
        let response = status(include_str!("testdata/tx_status_btc_to_tron_pending.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Pending,
                metadata: None,
                eta_in_seconds: Some(600),
            }
        );
    }

    #[test]
    fn test_map_swap_result_bnb_to_tron_pending() {
        let response = status(include_str!("testdata/tx_status_bnb_to_tron_pending.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Pending,
                metadata: Some(
                    TransactionSwapMetadata::new(
                        Chain::SmartChain.as_asset_id(),
                        BigUint::from(20000000000000000u64),
                        Chain::Tron.as_asset_id(),
                        BigUint::from(43070556u64),
                        SwapProvider::Thorchain
                    )
                    .with_referral_fee(Some(TransactionSwapReferralFee {
                        asset_id: Chain::Thorchain.as_asset_id(),
                        value: BigUint::from(15555500u64)
                    }))
                ),
                eta_in_seconds: Some(120),
            }
        );
    }

    #[test]
    fn test_map_swap_result_bnb_to_eth_usdt() {
        let response = status(include_str!("testdata/tx_status_bnb_to_eth_usdt.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(
                    TransactionSwapMetadata::new(
                        Chain::SmartChain.as_asset_id(),
                        BigUint::from(21300000000000000u64),
                        ETHEREUM_USDT_ASSET_ID.clone(),
                        BigUint::from(12973781u64),
                        SwapProvider::Thorchain
                    )
                    .with_referral_fee(Some(TransactionSwapReferralFee {
                        asset_id: Chain::Thorchain.as_asset_id(),
                        value: BigUint::from(15742200u64)
                    }))
                ),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_bnb_to_tron() {
        let response = status(include_str!("testdata/tx_status_bnb_to_tron.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(
                    TransactionSwapMetadata::new(
                        Chain::SmartChain.as_asset_id(),
                        BigUint::from(20000000000000000u64),
                        Chain::Tron.as_asset_id(),
                        BigUint::from(43070556u64),
                        SwapProvider::Thorchain
                    )
                    .with_referral_fee(Some(TransactionSwapReferralFee {
                        asset_id: Chain::Thorchain.as_asset_id(),
                        value: BigUint::from(15555500u64)
                    }))
                ),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_eth_usdt_to_rune() {
        let response = status(include_str!("testdata/tx_status_eth_usdt_to_rune.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(
                    TransactionSwapMetadata::new(
                        ETHEREUM_USDT_ASSET_ID.clone(),
                        BigUint::from(8366000000u64),
                        Chain::Thorchain.as_asset_id(),
                        BigUint::from(2096315169517u64),
                        SwapProvider::Thorchain
                    )
                    .with_referral_fee(Some(TransactionSwapReferralFee {
                        asset_id: ETHEREUM_USDT_ASSET_ID.clone(),
                        value: BigUint::from(41830000u64),
                    }))
                ),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_tcy_to_eth_usdt() {
        let response = status(include_str!("testdata/tx_status_tcy_to_eth_usdt.json"));

        assert_eq!(
            map_swap_result(&response),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata::new(
                    THORCHAIN_TCY_ASSET_ID.clone(),
                    BigUint::from(11921829956942u64),
                    ETHEREUM_USDT_ASSET_ID.clone(),
                    BigUint::from(3809626562u64),
                    SwapProvider::Thorchain
                )),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_btc_to_eth_usdc_refund() {
        let response = status(include_str!("testdata/tx_status_btc_to_eth_usdc_refund.json"));

        assert_eq!(map_swap_result(&response), SwapResult::from(SwapStatus::Refunded));
    }

    #[test]
    fn test_map_swap_result_direct_affiliate_payout() {
        let response = status(include_str!("testdata/tx_status_bnb_to_avax_direct_affiliate.json"));
        let metadata = map_swap_result(&response).metadata.unwrap();

        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: Chain::Thorchain.as_asset_id(),
                value: BigUint::from(683105u64),
            })
        );
    }

    #[test]
    fn test_map_swap_result_memo_affiliate_without_payout() {
        let response = status(include_str!("testdata/tx_status_bnb_to_rune_memo_affiliate.json"));
        let metadata = map_swap_result(&response).metadata.unwrap();

        assert_eq!(metadata.from_value, BigUint::from(10000000000000000u64));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: Chain::SmartChain.as_asset_id(),
                value: BigUint::from(50000000000000u64),
            })
        );
    }
}
