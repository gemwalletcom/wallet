use primitives::{TransactionSwapMetadata, TransactionSwapReferralFee, swap::THORCHAIN_REFERRAL_ADDRESS};

use super::THORChainNetwork;
use super::chain::ChainName;
use super::memo::ThorchainMemo;
use super::model::TransactionStatus;
use crate::SwapResult;

pub fn map_swap_result(response: &TransactionStatus, network: THORChainNetwork) -> SwapResult {
    let status = response.swap_status();
    let eta_in_seconds = response.eta_in_seconds();

    let Some(ref transaction) = response.tx else {
        return SwapResult { status, metadata: None, eta_in_seconds };
    };

    if ChainName::from_symbol(network, &transaction.chain).is_none() {
        return SwapResult { status, metadata: None, eta_in_seconds };
    }

    let from_coin = transaction.coins.first();
    let from_asset = from_coin.and_then(|c| c.asset_id(network));
    let from_value = from_coin.and_then(|c| c.native_value(network));

    let out_coin = response.destination_coin();
    let to_asset = out_coin.and_then(|c| c.asset_id(network));
    let to_value = out_coin.and_then(|c| c.native_value(network));

    let referral_fee = map_referral_fee(response, &transaction.memo, network);
    let metadata = match (from_asset, from_value, to_asset, to_value) {
        (Some(from_asset), Some(from_value), Some(to_asset), Some(to_value)) => Some(TransactionSwapMetadata::new(from_asset, from_value, to_asset, to_value, network.provider()).with_referral_fee(referral_fee)),
        _ => None,
    };

    SwapResult { status, metadata, eta_in_seconds }
}

fn map_referral_fee(response: &TransactionStatus, memo: &str, network: THORChainNetwork) -> Option<TransactionSwapReferralFee> {
    if ThorchainMemo::parse(memo)?.affiliate.as_deref() != Some(THORCHAIN_REFERRAL_ADDRESS) {
        return None;
    }
    let coin = response
        .out_txs
        .as_ref()?
        .iter()
        .filter(|transaction| transaction.to_address.as_deref() == Some(network.affiliate_collector_address()))
        .find_map(|transaction| transaction.coins.first())?;
    Some(TransactionSwapReferralFee {
        asset_id: network.chain().as_asset_id(),
        value: coin.amount.parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use primitives::{
        Chain, SwapProvider,
        asset_constants::{ARBITRUM_USDC_ASSET_ID, ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDT_ASSET_ID, THORCHAIN_TCY_ASSET_ID, TRON_USDT_ASSET_ID},
        swap::SwapStatus,
    };

    fn status(json: &str) -> TransactionStatus {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_map_swap_result_ltc_to_tron_usdt() {
        let response = status(include_str!("testdata/tx_status_ltc_to_tron_usdt.json"));

        assert_eq!(
            map_swap_result(&response, THORChainNetwork::Thorchain),
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
            map_swap_result(&response, THORChainNetwork::Thorchain),
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
            map_swap_result(&response, THORChainNetwork::Thorchain),
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
            map_swap_result(&response, THORChainNetwork::Thorchain),
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
            map_swap_result(&response, THORChainNetwork::Thorchain),
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
            map_swap_result(&response, THORChainNetwork::Thorchain),
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
            map_swap_result(&response, THORChainNetwork::Thorchain),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata::new(
                    ETHEREUM_USDT_ASSET_ID.clone(),
                    BigUint::from(8366000000u64),
                    Chain::Thorchain.as_asset_id(),
                    BigUint::from(2096315169517u64),
                    SwapProvider::Thorchain
                )),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_tcy_to_eth_usdt() {
        let response = status(include_str!("testdata/tx_status_tcy_to_eth_usdt.json"));

        assert_eq!(
            map_swap_result(&response, THORChainNetwork::Thorchain),
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
    fn test_map_swap_result_mayachain_refund() {
        let response = status(include_str!("testdata/transaction_status_mayachain_refund.json"));

        assert_eq!(
            map_swap_result(&response, THORChainNetwork::Mayachain),
            SwapResult {
                status: SwapStatus::Refunded,
                metadata: None,
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_mayachain_referral_fee() {
        let response = status(include_str!("testdata/transaction_status_mayachain_usdc_to_arb_usdc.json"));
        let metadata = map_swap_result(&response, THORChainNetwork::Mayachain).metadata.unwrap();

        assert_eq!(metadata.from_asset, ETHEREUM_USDC_ASSET_ID.clone());
        assert_eq!(metadata.from_value, BigUint::from(10000000000u64));
        assert_eq!(metadata.to_asset, ARBITRUM_USDC_ASSET_ID.clone());
        assert_eq!(metadata.to_value, BigUint::from(9844766159u64));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: Chain::Mayachain.as_asset_id(),
                value: BigUint::from(4553329330200u64),
            })
        );
    }

    #[test]
    fn test_map_swap_result_mayachain_pending_eta() {
        let response = status(include_str!("testdata/transaction_status_mayachain_pending_eta.json"));

        assert_eq!(
            map_swap_result(&response, THORChainNetwork::Mayachain),
            SwapResult {
                status: SwapStatus::Pending,
                metadata: None,
                eta_in_seconds: Some(300),
            }
        );
    }
}
