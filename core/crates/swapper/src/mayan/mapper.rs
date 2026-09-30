use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{
    Asset, AssetId, TransactionSwapMetadata, TransactionSwapReferralFee,
    swap::{EVM_REFERRAL_ADDRESS, HUNDRED_PERCENT_IN_BPS, SOLANA_REFERRAL_ADDRESS, SUI_REFERRAL_ADDRESS},
};

use crate::{SwapResult, SwapperProvider};

const MAYAN_SWIFT_V1_SERVICE: &str = "SWIFT_SWAP";
const MAX_TOKEN_DECIMALS: u32 = 36;

use super::{
    asset::asset_id_for_token,
    model::{MayanClientStatus, MayanTransactionResult},
    wormhole_chain,
};

pub fn map_swap_result(result: &MayanTransactionResult, output_decimals: Option<u32>) -> SwapResult {
    SwapResult {
        status: result.client_status.swap_status(),
        metadata: result.swap_metadata(output_decimals),
        eta_in_seconds: None,
    }
}

impl MayanTransactionResult {
    fn swap_metadata(&self, output_decimals: Option<u32>) -> Option<TransactionSwapMetadata> {
        if self.client_status == MayanClientStatus::InProgress {
            return None;
        }

        let from_chain = self.from_token_chain.parse::<u16>().ok().and_then(wormhole_chain::chain_from_id)?;
        let to_chain = self.to_token_chain.parse::<u16>().ok().and_then(wormhole_chain::chain_from_id)?;
        let from_asset = asset_id_for_token(from_chain, &self.from_token_address)?;
        let from_value = match self.from_amount64.as_deref() {
            Some(value) => value.parse::<BigUint>().ok()?,
            None => decimal_value(self.from_amount.as_deref()?, asset_decimals(&from_asset)?)?,
        };
        let to_asset = asset_id_for_token(to_chain, &self.to_token_address)?;
        let to_value = match self.to_amount64.as_deref() {
            Some(value) => value.parse::<BigUint>().ok()?,
            None => decimal_value(self.to_amount.as_deref()?, asset_decimals(&to_asset).or(output_decimals).or_else(|| self.min_amount_decimals())?)?,
        };

        let referral_fee = match self.client_status {
            MayanClientStatus::Completed => self.referral_fee(&from_asset, &from_value, &to_asset, &to_value),
            MayanClientStatus::InProgress | MayanClientStatus::Refunded => None,
        };
        Some(TransactionSwapMetadata::new(from_asset, from_value, to_asset, to_value, SwapperProvider::Mayan).with_referral_fee(referral_fee))
    }

    fn min_amount_decimals(&self) -> Option<u32> {
        let amount = self.min_amount_out.as_deref()?;
        let value = self.min_amount_out64.as_deref()?.parse::<BigUint>().ok().filter(|value| *value > BigUint::ZERO)?;
        (0..=MAX_TOKEN_DECIMALS).find(|decimals| decimal_value(amount, *decimals).as_ref() == Some(&value))
    }

    fn referral_fee(&self, from_asset: &AssetId, from_value: &BigUint, to_asset: &AssetId, to_value: &BigUint) -> Option<TransactionSwapReferralFee> {
        let referrer = self.referrer_address.as_deref()?;
        if ![EVM_REFERRAL_ADDRESS, SOLANA_REFERRAL_ADDRESS, SUI_REFERRAL_ADDRESS].iter().any(|address| address.eq_ignore_ascii_case(referrer)) {
            return None;
        }
        let referrer_bps = self.referrer_bps.filter(|bps| *bps > 0)?;
        let (asset_id, value) = match self.service.as_deref() {
            Some(MAYAN_SWIFT_V1_SERVICE) => {
                let net_bps = HUNDRED_PERCENT_IN_BPS.checked_sub(referrer_bps + self.mayan_bps.unwrap_or_default())?;
                (to_asset.clone(), to_value * referrer_bps / net_bps)
            }
            _ => (from_asset.clone(), from_value * referrer_bps / HUNDRED_PERCENT_IN_BPS),
        };
        Some(TransactionSwapReferralFee { asset_id, value })
    }
}

fn asset_decimals(asset_id: &AssetId) -> Option<u32> {
    asset_id.is_native().then(|| Asset::from_chain(asset_id.chain).decimals as u32)
}

fn decimal_value(amount: &str, decimals: u32) -> Option<BigUint> {
    BigNumberFormatter::value_from_amount_exact(amount, decimals).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{
        AssetId, Chain,
        asset_constants::{ARBITRUM_USDT_ASSET_ID, BASE_USDC_ASSET_ID, ETHEREUM_USDT_ASSET_ID, HYPERCORE_SPOT_USDC_ASSET_ID, POLYGON_USDT_ASSET_ID},
        swap::SwapStatus,
    };

    fn result(json: &str) -> MayanTransactionResult {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_map_completed_swap_metadata() {
        for (json, from_asset, from_value, to_asset, to_value, referral_fee) in [
            (
                include_str!("test/pol_to_bnb_swift.json"),
                AssetId::from_chain(Chain::Polygon),
                "212000000000000000000",
                AssetId::from_chain(Chain::SmartChain),
                "33060513057817862",
                Some((AssetId::from_chain(Chain::Polygon), "1060000000000000000")),
            ),
            (
                include_str!("test/bnb_to_mon_swift.json"),
                AssetId::from_chain(Chain::SmartChain),
                "120000000000000000",
                AssetId::from_chain(Chain::Monad),
                "3306576785321161654272",
                Some((AssetId::from_chain(Chain::SmartChain), "600000000000000")),
            ),
            (
                include_str!("test/bnb_to_base_swift_v1.json"),
                AssetId::from_chain(Chain::SmartChain),
                "13000000000000000",
                AssetId::from_chain(Chain::Base),
                "3834692054613521",
                Some((AssetId::from_chain(Chain::Base), "19367131588957")),
            ),
            (
                include_str!("test/hype_to_hypercore_usdc_mono_chain.json"),
                AssetId::from_chain(Chain::Hyperliquid),
                "154100000000000000000",
                HYPERCORE_SPOT_USDC_ASSET_ID.clone(),
                "14828488252",
                Some((AssetId::from_chain(Chain::Hyperliquid), "770500000000000000")),
            ),
            (
                include_str!("test/eth_to_arbitrum_usdt_swift_v1_decimal_output.json"),
                AssetId::from_chain(Chain::Ethereum),
                "1450000000000000",
                ARBITRUM_USDT_ASSET_ID.clone(),
                "4696204",
                Some((ARBITRUM_USDT_ASSET_ID.clone(), "23718")),
            ),
            (
                include_str!("test/eth_to_sui_swift.json"),
                AssetId::from_chain(Chain::Ethereum),
                "10000000000000000",
                AssetId::from_chain(Chain::Sui),
                "7534906306",
                Some((AssetId::from_chain(Chain::Ethereum), "50000000000000")),
            ),
            (
                include_str!("test/sol_to_eth_swift.json"),
                AssetId::from_chain(Chain::Solana),
                "16195149",
                AssetId::from_chain(Chain::Base),
                "599671067569648",
                None,
            ),
            (
                include_str!("test/usdc_to_brla_fast_mctp.json"),
                BASE_USDC_ASSET_ID.clone(),
                "21667710",
                AssetId::from_token(Chain::Polygon, "0xE6A537a407488807F0bbeb0038B79004f19DDDFb"),
                "111502625917703364196",
                None,
            ),
            (
                include_str!("test/usdt_to_owb_swift.json"),
                POLYGON_USDT_ASSET_ID.clone(),
                "35243141",
                AssetId::from_token(Chain::Base, "0xEF5997c2cf2f6c138196f8A6203afc335206b3c1"),
                "398724622644505839482",
                None,
            ),
        ] {
            let referral_fee = referral_fee.map(|(asset_id, value)| TransactionSwapReferralFee { asset_id, value: value.parse().unwrap() });
            assert_eq!(
                map_swap_result(&result(json), None),
                SwapResult {
                    status: SwapStatus::Completed,
                    metadata: Some(TransactionSwapMetadata::new(from_asset, from_value.parse().unwrap(), to_asset, to_value.parse().unwrap(), SwapperProvider::Mayan).with_referral_fee(referral_fee)),
                    eta_in_seconds: None,
                }
            );
        }
    }

    #[test]
    fn test_map_swap_result_with_output_token_decimals() {
        let result = result(include_str!("test/eth_to_base_aero_swift_v1_decimal_output.json"));
        let to_asset = AssetId::from_token(Chain::Base, "0x940181a94A35A4569E4529A3CDfB74e38FD98631");

        assert!(map_swap_result(&result, None).metadata.is_none());
        assert_eq!(
            map_swap_result(&result, Some(18)).metadata,
            Some(
                TransactionSwapMetadata::new(
                    ETHEREUM_USDT_ASSET_ID.clone(),
                    "280000000".parse().unwrap(),
                    to_asset.clone(),
                    "447930094673253720688".parse().unwrap(),
                    SwapperProvider::Mayan
                )
                .with_referral_fee(Some(TransactionSwapReferralFee {
                    asset_id: to_asset,
                    value: "2262273205420473336".parse().unwrap(),
                }))
            )
        );
    }

    #[test]
    fn test_map_swap_result_without_metadata() {
        assert_eq!(
            map_swap_result(&result(include_str!("test/mctp_pending.json")), None),
            SwapResult {
                status: SwapStatus::Pending,
                metadata: None,
                eta_in_seconds: None,
            }
        );

        let refunded = map_swap_result(&result(include_str!("test/swift_refunded.json")), None);
        assert_eq!(refunded.status, SwapStatus::Refunded);
        assert_eq!(refunded.metadata.and_then(|metadata| metadata.referral_fee), None);

        let invalid = MayanTransactionResult {
            from_amount64: Some("invalid".to_string()),
            ..result(include_str!("test/pol_to_bnb_swift.json"))
        };
        assert!(map_swap_result(&invalid, None).metadata.is_none());
    }

    #[test]
    fn test_map_swap_result_rejects_decimal_hyperevm_output() {
        assert_eq!(
            map_swap_result(&result(include_str!("test/hyperevm_to_solana_invalid_amount.json")), None),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: None,
                eta_in_seconds: None,
            }
        );
    }
}
