use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use gem_client::{Client, ClientExt, Target, build_path_with_query};
use primitives::{
    AssetId, Chain, SwapProvider,
    swap::{SwapPartnerTransaction, SwapReferralFee, SwapStatus},
};
use serde::{Deserialize, Serialize};

use crate::{
    SwapperError,
    fees::default_referral_fees,
    partner::{SwapPartnerCursor, SwapPartnerProvider, SwapPartnerTransactionsPage},
};

const PROXY_TON_ADDRESSES: [&str; 2] = ["EQBnGWMCf3-FZZq1W4IWcWiGAc3PHuZ0_H-7sad2oY00o83S", "EQCM3B12QK1e4yZSf8GtBRT0aLMNyEsBc_DhVfRRtOEffLez"];
const HISTORY_START: NaiveDate = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();
const WINDOW: Duration = Duration::days(7);
const LATEST_LOOKBACK: Duration = Duration::days(1);
const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

#[derive(Debug, Clone, Deserialize)]
pub struct FeeAccrualsResponse {
    pub operations: Vec<FeeAccrual>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeeAccrual {
    pub pool_tx_hash: String,
    pub wallet_tx_hash: String,
    pub wallet_address: String,
    pub destination_wallet_address: String,
    pub success: bool,
    pub asset0_address: String,
    pub asset0_amount: String,
    pub asset1_address: String,
    pub asset1_amount: String,
    pub fee_asset_address: String,
    pub referral_fee_amount: String,
}

#[derive(Debug, Clone, Serialize)]
struct FeeAccrualsQuery {
    referrer_address: String,
    since: String,
    until: String,
}

#[derive(Debug, Clone)]
struct FeeAccrualsTarget {
    query: FeeAccrualsQuery,
}

impl Target for FeeAccrualsTarget {
    fn path(&self) -> String {
        build_path_with_query("/v1/stats/fee_accruals", &self.query)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct StonfiPartnerCursor {
    since: DateTime<Utc>,
}

pub struct StonfiPartnerProvider<C: Client> {
    client: C,
    referrer: String,
}

impl<C: Client> StonfiPartnerProvider<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            referrer: default_referral_fees().ton.address,
        }
    }
}

fn map_asset_id(address: &str) -> AssetId {
    if PROXY_TON_ADDRESSES.contains(&address) {
        return AssetId::from_chain(Chain::Ton);
    }
    AssetId::from_token(Chain::Ton, address)
}

fn map_wallet_hash(value: &str) -> Option<String> {
    String::from_utf8(hex::decode(value).ok()?).ok()
}

pub fn map_partner_transaction(accrual: &FeeAccrual) -> Option<SwapPartnerTransaction> {
    let legs = [(&accrual.asset0_address, &accrual.asset0_amount), (&accrual.asset1_address, &accrual.asset1_amount)];
    let (from_address, from_amount) = legs.iter().find(|(_, amount)| !amount.starts_with('-'))?;
    let (to_address, to_amount) = legs.iter().find(|(_, amount)| amount.starts_with('-'))?;
    Some(SwapPartnerTransaction {
        provider: SwapProvider::StonfiV2,
        provider_transaction_id: accrual.pool_tx_hash.clone(),
        status: if accrual.success { SwapStatus::Completed } else { SwapStatus::Failed },
        from_address: accrual.wallet_address.clone(),
        to_address: accrual.destination_wallet_address.clone(),
        from_asset_id: map_asset_id(from_address),
        from_value: from_amount.to_string(),
        from_amount_usd: None,
        to_asset_id: map_asset_id(to_address),
        to_value: to_amount.trim_start_matches('-').to_string(),
        to_amount_usd: None,
        referral_fee: Some(SwapReferralFee {
            asset_id: map_asset_id(&accrual.fee_asset_address),
            value: accrual.referral_fee_amount.clone(),
            amount_usd: None,
        }),
        from_transaction_hash: map_wallet_hash(&accrual.wallet_tx_hash),
        to_transaction_hash: Some(accrual.pool_tx_hash.clone()),
    })
}

fn map_next_cursor(until: DateTime<Utc>, now: DateTime<Utc>) -> Result<SwapPartnerCursor, SwapperError> {
    if until < now {
        return Ok(SwapPartnerCursor::Next(serde_json::to_string(&StonfiPartnerCursor { since: until })?));
    }
    Ok(SwapPartnerCursor::Latest(serde_json::to_string(&StonfiPartnerCursor { since: now - LATEST_LOOKBACK })?))
}

#[async_trait]
impl<C: Client> SwapPartnerProvider for StonfiPartnerProvider<C> {
    fn name(&self) -> &'static str {
        "stonfi_v2"
    }

    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError> {
        let since = match cursor {
            Some(cursor) => serde_json::from_str::<StonfiPartnerCursor>(&cursor)?.since,
            None => HISTORY_START.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc(),
        };
        let now = Utc::now();
        let until = (since + WINDOW).min(now);
        let query = FeeAccrualsQuery {
            referrer_address: self.referrer.clone(),
            since: since.format(TIMESTAMP_FORMAT).to_string(),
            until: until.format(TIMESTAMP_FORMAT).to_string(),
        };
        let response: FeeAccrualsResponse = self.client.get(FeeAccrualsTarget { query }).await.map_err(SwapperError::from)?;
        Ok(SwapPartnerTransactionsPage {
            transactions: response.operations.iter().filter_map(map_partner_transaction).collect(),
            cursor: map_next_cursor(until, now)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use primitives::asset_constants::TON_USDT_ASSET_ID;

    use super::*;

    fn operations() -> Vec<FeeAccrual> {
        serde_json::from_str::<FeeAccrualsResponse>(include_str!("testdata/fee_accruals.json")).unwrap().operations
    }

    #[test]
    fn test_map_partner_transaction() {
        let operations = operations();
        let usdt_to_ton = map_partner_transaction(&operations[0]).unwrap();

        assert_eq!(usdt_to_ton.status, SwapStatus::Completed);
        assert_eq!((usdt_to_ton.from_asset_id, usdt_to_ton.from_value), (TON_USDT_ASSET_ID.clone(), "1980000".to_string()));
        assert_eq!((usdt_to_ton.to_asset_id, usdt_to_ton.to_value), (AssetId::from_chain(Chain::Ton), "1219582736".to_string()));
        let fee = usdt_to_ton.referral_fee.unwrap();
        assert_eq!((fee.asset_id, fee.value), (AssetId::from_chain(Chain::Ton), "6130405".to_string()));
        assert_eq!(usdt_to_ton.from_transaction_hash.as_deref(), Some("3a0149e1caa26385206d379aa6a72de1dfefca2618901fe9493b09e599569264"));

        let ton_to_token = map_partner_transaction(&operations[1]).unwrap();
        assert_eq!(ton_to_token.from_asset_id, AssetId::from_chain(Chain::Ton));
        assert_eq!(ton_to_token.to_value, "4270903568601535");
    }

    #[test]
    fn test_map_next_cursor() {
        let now = "2026-09-29T05:00:00Z".parse::<DateTime<Utc>>().unwrap();

        assert_eq!(
            map_next_cursor("2026-09-20T00:00:00Z".parse().unwrap(), now).unwrap(),
            SwapPartnerCursor::Next(r#"{"since":"2026-09-20T00:00:00Z"}"#.to_string())
        );
        assert_eq!(map_next_cursor(now, now).unwrap(), SwapPartnerCursor::Latest(r#"{"since":"2026-09-28T05:00:00Z"}"#.to_string()));
    }
}
