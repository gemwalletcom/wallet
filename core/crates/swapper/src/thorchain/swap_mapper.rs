use primitives::TransactionSwapMetadata;

use super::THORChainNetwork;
use super::chain::ChainName;
use super::model::TransactionStatus;
use crate::SwapResult;

pub fn map_swap_result(response: &TransactionStatus, network: THORChainNetwork) -> SwapResult {
    let status = response.swap_status();
    let eta_in_seconds = response.eta_in_seconds();

    let Some(ref tx) = response.tx else {
        return SwapResult {
            status,
            metadata: None,
            eta_in_seconds,
        };
    };

    let Some(chain) = ChainName::from_symbol(network, &tx.chain).map(|n| n.chain()) else {
        return SwapResult {
            status,
            metadata: None,
            eta_in_seconds,
        };
    };

    let from_coin = tx.coins.first();
    let from_asset = from_coin.and_then(|c| c.resolve_asset_id(network));
    let from_value = from_coin.and_then(|c| c.native_value(chain));

    let out_coin = response.destination_coin();
    let to_asset = out_coin.and_then(|c| c.resolve_asset_id(network));
    let to_value = out_coin.and_then(|c| to_asset.as_ref().and_then(|a| c.native_value(a.chain)));

    let metadata = match (from_asset, from_value, to_asset, to_value) {
        (Some(from_asset), Some(from_value), Some(to_asset), Some(to_value)) => Some(TransactionSwapMetadata {
            from_asset,
            from_value,
            to_asset,
            to_value,
            provider: Some(network.provider().as_ref().to_string()),
        }),
        _ => None,
    };

    SwapResult { status, metadata, eta_in_seconds }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use primitives::{
        Chain,
        asset_constants::{ETHEREUM_USDT_ASSET_ID, THORCHAIN_TCY_ASSET_ID, TRON_USDT_ASSET_ID},
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
                metadata: Some(TransactionSwapMetadata {
                    from_asset: Chain::Litecoin.as_asset_id(),
                    from_value: BigUint::from(160661010u64),
                    to_asset: TRON_USDT_ASSET_ID.clone(),
                    to_value: BigUint::from(79158429u64),
                    provider: Some("thorchain".to_string()),
                }),
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
                metadata: Some(TransactionSwapMetadata {
                    from_asset: Chain::Litecoin.as_asset_id(),
                    from_value: BigUint::from(5000000u64),
                    to_asset: Chain::Ethereum.as_asset_id(),
                    to_value: BigUint::from(1243680000000000u64),
                    provider: Some("thorchain".to_string()),
                }),
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
                metadata: Some(TransactionSwapMetadata {
                    from_asset: Chain::SmartChain.as_asset_id(),
                    from_value: BigUint::from(20000000000000000u64),
                    to_asset: Chain::Tron.as_asset_id(),
                    to_value: BigUint::from(43070556u64),
                    provider: Some("thorchain".to_string()),
                }),
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
                metadata: Some(TransactionSwapMetadata {
                    from_asset: Chain::SmartChain.as_asset_id(),
                    from_value: BigUint::from(21300000000000000u64),
                    to_asset: ETHEREUM_USDT_ASSET_ID.clone(),
                    to_value: BigUint::from(12973781u64),
                    provider: Some("thorchain".to_string()),
                }),
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
                metadata: Some(TransactionSwapMetadata {
                    from_asset: Chain::SmartChain.as_asset_id(),
                    from_value: BigUint::from(20000000000000000u64),
                    to_asset: Chain::Tron.as_asset_id(),
                    to_value: BigUint::from(43070556u64),
                    provider: Some("thorchain".to_string()),
                }),
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
                metadata: Some(TransactionSwapMetadata {
                    from_asset: ETHEREUM_USDT_ASSET_ID.clone(),
                    from_value: BigUint::from(8366000000u64),
                    to_asset: Chain::Thorchain.as_asset_id(),
                    to_value: BigUint::from(2096315169517u64),
                    provider: Some("thorchain".to_string()),
                }),
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
                metadata: Some(TransactionSwapMetadata {
                    from_asset: THORCHAIN_TCY_ASSET_ID.clone(),
                    from_value: BigUint::from(11921829956942u64),
                    to_asset: ETHEREUM_USDT_ASSET_ID.clone(),
                    to_value: BigUint::from(3809626562u64),
                    provider: Some("thorchain".to_string()),
                }),
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
                status: SwapStatus::Failed,
                metadata: None,
                eta_in_seconds: None,
            }
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
