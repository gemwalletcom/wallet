use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{Asset, TransactionSwapMetadata};

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
        let from_value = if from_asset.is_native() {
            let decimals = u32::try_from(Asset::from_chain(from_chain).decimals).ok()?;
            BigNumberFormatter::value_from_amount(self.from_amount.as_deref()?, decimals).ok()?
        } else {
            self.from_amount64.as_deref()?.parse::<BigUint>().ok()?.to_string()
        };
        let to_asset = asset_id_for_token(to_chain, &self.to_token_address)?;
        let to_value = self.to_amount64.as_deref()?.parse::<BigUint>().ok()?.to_string();

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
    use primitives::{AssetId, Chain, asset_constants::POLYGON_USDT_ASSET_ID, swap::SwapStatus};

    fn result(json: &str) -> MayanTransactionResult {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_map_swap_result() {
        let missing_to_amount64 = map_swap_result(&result(include_str!("test/eth_to_sui_swift.json")));
        assert_eq!(missing_to_amount64.status, SwapStatus::Completed);
        assert!(missing_to_amount64.metadata.is_none());

        let invalid_from_amount = MayanTransactionResult {
            from_amount: Some("invalid".to_string()),
            ..result(include_str!("test/pol_to_bnb_swift.json"))
        };
        assert_eq!(
            map_swap_result(&invalid_from_amount),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: None,
                eta_in_seconds: None,
            }
        );

        assert_eq!(
            map_swap_result(&result(include_str!("test/pol_to_bnb_swift.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Polygon),
                    from_value: "212000000000000000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::SmartChain),
                    to_value: "33060513057817862".to_string(),
                    provider: Some("mayan".to_string()),
                }),
                eta_in_seconds: None,
            }
        );
        assert_eq!(
            map_swap_result(&result(include_str!("test/bnb_to_mon_swift.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::SmartChain),
                    from_value: "120000000000000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::Monad),
                    to_value: "3306576785321161654272".to_string(),
                    provider: Some("mayan".to_string()),
                }),
                eta_in_seconds: None,
            }
        );
        assert_eq!(
            map_swap_result(&result(include_str!("test/usdt_to_owb_swift.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: POLYGON_USDT_ASSET_ID.clone(),
                    from_value: "35245466".to_string(),
                    to_asset: AssetId::from_token(Chain::Base, "0xEF5997c2cf2f6c138196f8A6203afc335206b3c1"),
                    to_value: "398724622644505839482".to_string(),
                    provider: Some("mayan".to_string()),
                }),
                eta_in_seconds: None,
            }
        );
        assert_eq!(
            map_swap_result(&result(include_str!("test/btcbr_to_radr_swift.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_token(Chain::SmartChain, "0x0cF8e180350253271f4b917CcFb0aCCc4862F262"),
                    from_value: "10686571736749000000".to_string(),
                    to_asset: AssetId::from_token(Chain::Solana, "CzFvsLdUazabdiu9TYXujj4EY495fG7VgJJ3vQs6bonk"),
                    to_value: "278080608518046".to_string(),
                    provider: Some("mayan".to_string()),
                }),
                eta_in_seconds: None,
            }
        );

        let pending = map_swap_result(&result(include_str!("test/mctp_pending.json")));
        assert_eq!(pending.status, SwapStatus::Pending);
        assert!(pending.metadata.is_none());

        let refunded = map_swap_result(&result(include_str!("test/swift_refunded.json")));
        assert_eq!(refunded.status, SwapStatus::Failed);
        assert!(refunded.metadata.is_none());
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
