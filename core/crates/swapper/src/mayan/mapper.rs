use num_bigint::BigUint;
use primitives::TransactionSwapMetadata;

use crate::{SwapResult, SwapperProvider};

use super::{
    asset::asset_id_for_token,
    model::{MayanClientStatus, MayanTransactionResult},
    wormhole_chain,
};

pub fn map_swap_result(result: &MayanTransactionResult) -> SwapResult {
    SwapResult {
        status: result.client_status.swap_status(),
        metadata: result.swap_metadata(),
        eta_in_seconds: None,
    }
}

impl MayanTransactionResult {
    fn swap_metadata(&self) -> Option<TransactionSwapMetadata> {
        if self.client_status == MayanClientStatus::InProgress {
            return None;
        }

        let from_chain = self.from_token_chain.parse::<u16>().ok().and_then(wormhole_chain::chain_from_id)?;
        let to_chain = self.to_token_chain.parse::<u16>().ok().and_then(wormhole_chain::chain_from_id)?;
        let from_asset = asset_id_for_token(from_chain, &self.from_token_address)?;
        let from_value = self.from_amount64.as_deref()?.parse::<BigUint>().ok()?;
        let to_asset = asset_id_for_token(to_chain, &self.to_token_address)?;
        let to_value = self.to_amount64.as_deref()?.parse::<BigUint>().ok()?;

        Some(TransactionSwapMetadata {
            from_asset,
            from_value,
            to_asset,
            to_value,
            provider: Some(SwapperProvider::Mayan.as_ref().to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{
        AssetId, Chain,
        asset_constants::{BASE_USDC_ASSET_ID, POLYGON_USDT_ASSET_ID},
        swap::SwapStatus,
    };

    fn result(json: &str) -> MayanTransactionResult {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_map_completed_swap_metadata() {
        for (json, from_asset, from_value, to_asset, to_value) in [
            (
                include_str!("test/pol_to_bnb_swift.json"),
                AssetId::from_chain(Chain::Polygon),
                "212000000000000000000",
                AssetId::from_chain(Chain::SmartChain),
                "33060513057817862",
            ),
            (
                include_str!("test/bnb_to_mon_swift.json"),
                AssetId::from_chain(Chain::SmartChain),
                "120000000000000000",
                AssetId::from_chain(Chain::Monad),
                "3306576785321161654272",
            ),
            (
                include_str!("test/sol_to_eth_swift.json"),
                AssetId::from_chain(Chain::Solana),
                "16195149",
                AssetId::from_chain(Chain::Base),
                "599671067569648",
            ),
            (
                include_str!("test/usdc_to_brla_fast_mctp.json"),
                BASE_USDC_ASSET_ID.clone(),
                "21667710",
                AssetId::from_token(Chain::Polygon, "0xE6A537a407488807F0bbeb0038B79004f19DDDFb"),
                "111502625917703364196",
            ),
            (
                include_str!("test/usdt_to_owb_swift.json"),
                POLYGON_USDT_ASSET_ID.clone(),
                "35243141",
                AssetId::from_token(Chain::Base, "0xEF5997c2cf2f6c138196f8A6203afc335206b3c1"),
                "398724622644505839482",
            ),
        ] {
            assert_eq!(
                map_swap_result(&result(json)),
                SwapResult {
                    status: SwapStatus::Completed,
                    metadata: Some(TransactionSwapMetadata {
                        from_asset,
                        from_value: from_value.parse().unwrap(),
                        to_asset,
                        to_value: to_value.parse().unwrap(),
                        provider: Some("mayan".to_string()),
                    }),
                    eta_in_seconds: None,
                }
            );
        }
    }

    #[test]
    fn test_map_swap_result_without_metadata() {
        for (json, status) in [
            (include_str!("test/eth_to_sui_swift.json"), SwapStatus::Completed),
            (include_str!("test/mctp_pending.json"), SwapStatus::Pending),
            (include_str!("test/swift_refunded.json"), SwapStatus::Failed),
        ] {
            assert_eq!(
                map_swap_result(&result(json)),
                SwapResult {
                    status,
                    metadata: None,
                    eta_in_seconds: None,
                }
            );
        }

        let invalid = MayanTransactionResult {
            from_amount64: Some("invalid".to_string()),
            ..result(include_str!("test/pol_to_bnb_swift.json"))
        };
        assert!(map_swap_result(&invalid).metadata.is_none());
    }

    #[test]
    fn test_map_swap_result_rejects_decimal_hyperevm_output() {
        assert_eq!(
            map_swap_result(&result(include_str!("test/hyperevm_to_solana_invalid_amount.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: None,
                eta_in_seconds: None,
            }
        );
    }
}
