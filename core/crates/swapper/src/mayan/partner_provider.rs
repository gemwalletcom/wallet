use std::{fmt::Debug, str::FromStr};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use gem_client::Client;
use num_bigint::BigUint;
use primitives::{
    AssetId, SwapProvider,
    swap::{SwapPartnerTransaction, SwapReferralFee},
};
use serde::{Deserialize, Serialize};

use super::{
    asset::asset_id_for_token,
    client::MayanClient,
    model::{MayanSwapsQuery, MayanTransactionResult},
    wormhole_chain::chain_from_id,
};
use crate::{
    SwapperError,
    fees::default_referral_fees,
    partner::{SwapPartnerCursor, SwapPartnerProvider, SwapPartnerTransactionsPage},
};

const SWAPS_LIMIT: usize = 100;
const UPDATED_OVERLAP: Duration = Duration::hours(1);
const BPS_DIVISOR: u32 = 10_000;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MayanPartnerCursor {
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_after: Option<DateTime<Utc>>,
}

pub struct MayanPartnerProvider<C: Client + Clone + Send + Sync + Debug + 'static> {
    client: MayanClient<C>,
    referrers: Vec<String>,
}

impl<C: Client + Clone + Send + Sync + Debug + 'static> MayanPartnerProvider<C> {
    pub fn new(client: C) -> Self {
        let fees = default_referral_fees();
        Self {
            client: MayanClient::new(client),
            referrers: vec![fees.evm.address, fees.solana.address, fees.sui.address],
        }
    }
}

fn map_asset_id(chain_id: &str, token_address: &str) -> Option<AssetId> {
    asset_id_for_token(chain_from_id(chain_id.parse().ok()?)?, token_address)
}

pub fn map_partner_transaction(swap: &MayanTransactionResult, referrers: &[String]) -> Option<SwapPartnerTransaction> {
    let from_asset_id = map_asset_id(&swap.from_token_chain, &swap.from_token_address)?;
    let from_value = swap.from_amount64.clone()?;
    let to_asset_id = map_asset_id(&swap.to_token_chain, &swap.to_token_address)?;
    let fee_bps = swap.referrer_bps.filter(|bps| *bps > 0);
    let from_amount_usd = swap.referrer_fee_usd.zip(fee_bps).map(|(fee_usd, bps)| fee_usd * f64::from(BPS_DIVISOR) / f64::from(bps));
    let to_amount_usd = swap.to_amount.as_ref().and_then(|amount| amount.parse::<f64>().ok()).zip(swap.to_token_price).map(|(amount, price)| amount * price);
    let referral_fee = swap
        .referrer_address
        .as_ref()
        .filter(|address| referrers.iter().any(|referrer| referrer.eq_ignore_ascii_case(address)))
        .and(fee_bps)
        .and_then(|bps| {
            Some(SwapReferralFee {
                asset_id: from_asset_id.clone(),
                value: (BigUint::from_str(&from_value).ok()? * bps / BPS_DIVISOR).to_string(),
                amount_usd: swap.referrer_fee_usd,
            })
        });
    Some(SwapPartnerTransaction {
        provider: SwapProvider::Mayan,
        provider_transaction_id: swap.source_tx_hash.clone()?,
        status: swap.client_status.swap_status(),
        from_address: swap.trader.clone()?,
        to_address: swap.dest_address.clone()?,
        from_asset_id,
        from_value,
        from_amount_usd,
        to_asset_id,
        to_value: swap.to_amount64.clone()?,
        to_amount_usd,
        referral_fee,
        from_transaction_hash: swap.source_tx_hash.clone(),
        to_transaction_hash: swap.fulfill_tx_hash.clone().or_else(|| swap.redeem_tx_hash.clone()),
    })
}

#[async_trait]
impl<C: Client + Clone + Send + Sync + Debug + 'static> SwapPartnerProvider for MayanPartnerProvider<C> {
    fn name(&self) -> &str {
        SwapProvider::Mayan.id()
    }

    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError> {
        let cursor: MayanPartnerCursor = cursor.map(|cursor| serde_json::from_str(&cursor)).transpose()?.unwrap_or_default();
        let updated_since = cursor.updated_after.map(|updated_after| updated_after - UPDATED_OVERLAP);
        let mut latest = cursor.updated_after;
        let mut transactions = Vec::new();
        for referrer in &self.referrers {
            let mut offset = 0;
            loop {
                let query = MayanSwapsQuery {
                    referrer_address: referrer.clone(),
                    limit: SWAPS_LIMIT,
                    offset,
                };
                let swaps = self.client.get_swaps(query).await?.data;
                for swap in swaps.iter().filter(|swap| updated_since.zip(swap.status_updated_at).is_none_or(|(since, updated_at)| updated_at >= since)) {
                    let details = self.client.get_transaction_status(&swap.source_tx_hash).await?;
                    transactions.extend(map_partner_transaction(&details, &self.referrers));
                    latest = latest.max(swap.status_updated_at);
                }
                if swaps.len() < SWAPS_LIMIT {
                    break;
                }
                offset += SWAPS_LIMIT;
            }
        }
        Ok(SwapPartnerTransactionsPage {
            transactions,
            cursor: SwapPartnerCursor::Latest(serde_json::to_string(&MayanPartnerCursor { updated_after: latest })?),
        })
    }
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, swap::SwapStatus};

    use super::*;

    fn swaps() -> Vec<MayanTransactionResult> {
        serde_json::from_str(include_str!("test/partner_swaps.json")).unwrap()
    }

    fn referrers() -> Vec<String> {
        vec!["0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".to_string(), "5fmLrs2GuhfDP1B51ziV5Kd1xtAr9rw1jf3aQ4ihZ2gy".to_string()]
    }

    #[test]
    fn test_map_partner_transaction() {
        let swaps = swaps();
        let transaction = map_partner_transaction(&swaps[0], &referrers()).unwrap();

        assert_eq!(transaction.status, SwapStatus::Completed);
        assert_eq!(transaction.from_address, "0x7EedbFd6cf64aC9eFA1dBE137dD33A7f0E195719");
        assert_eq!((transaction.from_asset_id, transaction.from_value), (AssetId::from_chain(Chain::Ethereum), "52248137728882455".to_string()));
        assert_eq!((transaction.to_asset_id, transaction.to_value), (AssetId::from_chain(Chain::Hyperliquid), "1596017864173172992".to_string()));
        let fee = transaction.referral_fee.unwrap();
        assert_eq!((fee.asset_id, fee.value), (AssetId::from_chain(Chain::Ethereum), "261240688644412".to_string()));
        assert!((transaction.from_amount_usd.unwrap() - 139.19316012396156).abs() < 1e-9);
        assert_eq!(transaction.to_transaction_hash.as_deref(), Some("0x0aeac6a94c2b428c17bd5a8d18835152d7a2e109ceeb6290838fce0e309055c7"));

        let solana = map_partner_transaction(&swaps[1], &referrers()).unwrap();
        assert_eq!(solana.from_asset_id, AssetId::from_token(Chain::Solana, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"));
        assert_eq!(solana.to_asset_id.chain, Chain::SmartChain);

        assert_eq!(map_partner_transaction(&swaps[2], &referrers()).unwrap().status, SwapStatus::Refunded);
        assert_eq!(map_partner_transaction(&swaps[0], &[]).unwrap().referral_fee, None);
    }
}
