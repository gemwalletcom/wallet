use num_bigint::BigUint;
use num_traits::ToPrimitive;
use primitives::swap::{CHAINFLIP_REFERRAL_ADDRESS, SwapStatus};
use primitives::{AssetId, TransactionSwapMetadata, TransactionSwapReferralFee};
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

use super::AssetsResponse;
use crate::chainflip::{chain::ChainflipChain, client::chainflip_asset_to_asset_id};
use crate::{SwapResult, SwapperProvider};

const BROKER_FEE_TYPE: &str = "broker";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusByTransactionHashRequest {
    pub transaction_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct BrokerStatusResponse {
    pub status: BrokerSwapStatus,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerSwapStatus {
    state: String,
    source_asset: String,
    destination_asset: String,
    deposit_channel: Option<BrokerDepositChannel>,
    estimated_duration_seconds: Option<f64>,
    #[serde(default)]
    fees: Vec<BrokerSwapFee>,
    deposit: Option<BrokerAmount>,
    swap: Option<BrokerSwapOutput>,
    refund_egress: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrokerDepositChannel {
    broker_commission_bps: u32,
    #[serde(default)]
    affiliate_brokers: Vec<BrokerAffiliate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrokerAffiliate {
    account: String,
    commission_bps: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrokerSwapFee {
    #[serde(rename = "type")]
    fee_type: String,
    asset: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    amount_native: BigUint,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrokerAmount {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    amount_native: BigUint,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrokerSwapOutput {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    swapped_output_amount_native: BigUint,
}

impl BrokerSwapStatus {
    fn swap_status(&self) -> SwapStatus {
        match self.state.as_str() {
            "completed" if self.refund_egress.is_some() => SwapStatus::Refunded,
            "completed" => SwapStatus::Completed,
            "failed" => SwapStatus::Failed,
            _ => SwapStatus::Pending,
        }
    }

    fn referral_fee(&self, assets: &AssetsResponse) -> Option<TransactionSwapReferralFee> {
        let channel = self.deposit_channel.as_ref()?;
        let referral = channel.affiliate_brokers.iter().find(|broker| broker.account == CHAINFLIP_REFERRAL_ADDRESS && broker.commission_bps > 0)?;
        let total_bps = channel.broker_commission_bps + channel.affiliate_brokers.iter().map(|broker| broker.commission_bps).sum::<u32>();
        let fee = self.fees.iter().find(|fee| fee.fee_type == BROKER_FEE_TYPE)?;
        Some(TransactionSwapReferralFee {
            asset_id: asset_id(assets, &fee.asset)?,
            value: &fee.amount_native * referral.commission_bps / total_bps,
        })
    }

    pub fn swap_result(&self, assets: &AssetsResponse) -> SwapResult {
        let status = self.swap_status();
        if status == SwapStatus::Pending {
            let eta_in_seconds = self.estimated_duration_seconds.filter(|seconds| seconds.is_finite() && *seconds > 0.0).and_then(|seconds| seconds.ceil().to_u32());
            return SwapResult { status, metadata: None, eta_in_seconds };
        }

        let metadata = (|| {
            let from_asset = asset_id(assets, &self.source_asset)?;
            let to_asset = asset_id(assets, &self.destination_asset)?;
            let from_value = self.deposit.as_ref()?.amount_native.clone();
            let to_value = self.swap.as_ref().map(|swap| swap.swapped_output_amount_native.clone()).unwrap_or_default();
            Some(TransactionSwapMetadata::new(from_asset, from_value, to_asset, to_value, SwapperProvider::Chainflip).with_referral_fee(self.referral_fee(assets)))
        })();

        SwapResult { status, metadata, eta_in_seconds: None }
    }
}

fn asset_id(assets: &AssetsResponse, id: &str) -> Option<AssetId> {
    let asset = assets.assets.iter().find(|asset| asset.id == id)?;
    let chain = asset.network.parse::<ChainflipChain>().ok()?.to_chain();
    chainflip_asset_to_asset_id(chain, &asset.ticker)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    fn status(json: &str) -> BrokerSwapStatus {
        serde_json::from_str::<BrokerStatusResponse>(json).unwrap().status
    }

    #[test]
    fn test_swap_result_completed() {
        let status = status(include_str!("./test/status_dot_to_sol.json"));
        let result = status.swap_result(&AssetsResponse::mock());

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::Polkadot));
        assert_eq!(metadata.from_value, BigUint::from(100_000_000_000u64));
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Solana));
        assert_eq!(metadata.to_value, BigUint::from(102_937_756u64));
        let referral_fee = metadata.referral_fee.unwrap();
        assert_eq!(referral_fee.asset_id, primitives::known_assets::ETHEREUM_USDC.id.clone());
        assert_eq!(referral_fee.value, BigUint::from(47_576u32));
    }

    #[test]
    fn test_swap_result_pending_and_refunded() {
        let pending = status(r#"{"id":1,"status":{"state":"receiving","sourceAsset":"dot.hub","destinationAsset":"eth.eth","estimatedDurationSeconds":102.26}}"#);
        assert_eq!(
            pending.swap_result(&AssetsResponse::mock()),
            SwapResult {
                status: SwapStatus::Pending,
                metadata: None,
                eta_in_seconds: Some(103),
            }
        );

        let refunded = status(r#"{"id":1,"status":{"state":"completed","sourceAsset":"dot.hub","destinationAsset":"eth.eth","deposit":{"amountNative":"50000000000"},"refundEgress":{"amountNative":"49800000000"}}}"#);
        assert_eq!(refunded.swap_result(&AssetsResponse::mock()).status, SwapStatus::Refunded);
    }
}
