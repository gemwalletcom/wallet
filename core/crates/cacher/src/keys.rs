use config_keys::{RateLimitKey, RateLimitWindow};
use primitives::{SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};

const SECONDS_PER_YEAR: u64 = 365 * SECONDS_PER_DAY;

pub enum CacheKey<'a> {
    ReferralIpCheck(&'a str),

    InactiveDeviceObserver(&'a str),
    DeviceStreamEvents(&'a str, u64),

    FetchCoinAddresses(&'a str, &'a str),
    FetchTokenAddresses(&'a str, &'a str),
    FetchNftAssetsAddresses(&'a str, &'a str),
    FetchAddressTransactions(&'a str, &'a str),

    FetchAssets(&'a str),
    FetchNftAsset(&'a str),
    Price(&'a str),
    PriceMetadata(&'a str, u64),
    PriceMissingMapping(&'a str, &'a str, u64),

    FiatRates,
    FiatQuote(i32, i32, &'a str),
    FiatIpCheck(&'a str),

    RateLimit(RateLimitKey, &'a str, RateLimitWindow),

    AuthNonce(&'a str, &'a str),

    AddressStatus(&'a str, &'a str),

    JobStatus(&'a str),

    Markets,
    ObservedAssets,

    SwapDepositAddresses(&'a str),
    SwapSendAddresses(&'a str),

    ChartsHistory(&'a str),

    AlerterStakeRewards(&'a str, &'a str),

    PerpetualTrackedAddresses(&'a str),
    PerpetualActiveAddresses(&'a str),
    PerpetualPriorityAddresses(&'a str),
    PerpetualObserverCheckpoint(&'a str, &'a str),

    FetchTransaction(&'a str, &'a str),
    PendingTransactions(&'a str),
    TransactionFeeEstimates(&'a str),
    TransactionFeeEstimatesFresh(&'a str),

    ScanSafe(&'a str, &'a str, u64),
}

impl CacheKey<'_> {
    pub fn key(&self) -> String {
        match self {
            Self::ReferralIpCheck(ip_address) => format!("referral:ip_check:{}", ip_address),
            Self::InactiveDeviceObserver(device_id) => format!("device:inactive_observer:{}", device_id),
            Self::DeviceStreamEvents(device_id, _) => format!("stream:device:events:{}", device_id),
            Self::FetchCoinAddresses(chain, address) => format!("fetch:coin_addresses:{}:{}", chain, address),
            Self::FetchTokenAddresses(chain, address) => format!("fetch:token_addresses:{}:{}", chain, address),
            Self::FetchNftAssetsAddresses(chain, address) => format!("fetch:nft_assets_addresses:{}:{}", chain, address),
            Self::FetchAddressTransactions(chain, address) => format!("fetch:address_transactions:{}:{}", chain, address),
            Self::FetchTransaction(chain, hash) => format!("fetch:transaction:{}:{}", chain, hash),
            Self::FetchAssets(asset_id) => format!("fetch:assets:{}", asset_id),
            Self::FetchNftAsset(asset_id) => format!("fetch:nft_asset:{}", asset_id),
            Self::Price(asset_id) => format!("prices:{}", asset_id),
            Self::PriceMetadata(id, _) => format!("prices:metadata:{}", id),
            Self::PriceMissingMapping(provider, id, _) => format!("prices:missing_mapping:{}:{}", provider, id),
            Self::FiatRates => "fiat:rates".to_string(),
            Self::FiatQuote(device_id, wallet_id, quote_id) => format!("fiat:quote:{}:{}:{}", device_id, wallet_id, quote_id),
            Self::FiatIpCheck(ip_address) => format!("fiat:ip_check:{}", ip_address),
            Self::RateLimit(key, scope, window) => format!("rate_limit:{}:{}:{}", key.as_ref(), window.as_ref(), scope),
            Self::AuthNonce(device_id, nonce) => format!("auth:nonce:{}:{}", device_id, nonce),
            Self::AddressStatus(chain, address) => format!("address:status:{}:{}", chain, address),
            Self::JobStatus(name) => format!("jobs:status:{}", name),
            Self::Markets => "markets:markets".to_string(),
            Self::ObservedAssets => "pricer:observed_assets".to_string(),
            Self::SwapDepositAddresses(provider) => format!("swap:deposit_addresses:{}", provider),
            Self::SwapSendAddresses(provider) => format!("swap:send_addresses:{}", provider),
            Self::ChartsHistory(provider) => format!("charts:history:{}", provider),
            Self::AlerterStakeRewards(chain, address) => format!("alerter:stake_rewards:{}:{}", chain, address),
            Self::PerpetualTrackedAddresses(chain) => format!("perpetual:tracked_addresses:{}", chain),
            Self::PerpetualActiveAddresses(chain) => format!("perpetual:active_addresses:{}", chain),
            Self::PerpetualPriorityAddresses(chain) => format!("perpetual:priority_addresses:{}", chain),
            Self::PerpetualObserverCheckpoint(chain, address) => format!("perpetual:last_seen:{}:{}", chain, address),
            Self::PendingTransactions(chain) => format!("transactions:pending:{}", chain),
            Self::TransactionFeeEstimates(chain) => format!("transactions:fee_estimates:{}", chain),
            Self::TransactionFeeEstimatesFresh(chain) => format!("transactions:fee_estimates:fresh:{}", chain),
            Self::ScanSafe(scan_type, target, _) => format!("scan:safe:{}:{}", scan_type, target),
        }
    }

    pub fn ttl(&self) -> u64 {
        match self {
            Self::ReferralIpCheck(_) => SECONDS_PER_DAY,
            Self::InactiveDeviceObserver(_) => 30 * SECONDS_PER_DAY,
            Self::DeviceStreamEvents(_, ttl) => *ttl,
            Self::FetchCoinAddresses(_, _) => 7 * SECONDS_PER_DAY,
            Self::FetchTokenAddresses(_, _) => 30 * SECONDS_PER_DAY,
            Self::FetchNftAssetsAddresses(_, _) => 30 * SECONDS_PER_DAY,
            Self::FetchAddressTransactions(_, _) => 30 * SECONDS_PER_DAY,
            Self::FetchTransaction(_, _) => 30 * SECONDS_PER_DAY,
            Self::FetchAssets(_) => 30 * SECONDS_PER_DAY,
            Self::FetchNftAsset(_) => SECONDS_PER_HOUR,
            Self::Price(_) => 30 * SECONDS_PER_DAY,
            Self::PriceMetadata(_, ttl) | Self::PriceMissingMapping(_, _, ttl) => *ttl,
            Self::FiatRates => SECONDS_PER_DAY,
            Self::FiatQuote(_, _, _) => 15 * SECONDS_PER_MINUTE,
            Self::FiatIpCheck(_) => SECONDS_PER_DAY,
            Self::RateLimit(_, _, window) => window.duration().as_secs(),
            Self::AuthNonce(_, _) => 5 * SECONDS_PER_MINUTE,
            Self::AddressStatus(_, _) => SECONDS_PER_YEAR,
            Self::JobStatus(_) => 7 * SECONDS_PER_DAY,
            Self::Markets => SECONDS_PER_DAY,
            Self::ObservedAssets => 2 * SECONDS_PER_MINUTE,
            Self::SwapDepositAddresses(_) => 7 * SECONDS_PER_DAY,
            Self::SwapSendAddresses(_) => 7 * SECONDS_PER_DAY,
            Self::ChartsHistory(_) => 10 * 365 * SECONDS_PER_DAY,
            Self::AlerterStakeRewards(_, _) => 30 * SECONDS_PER_DAY,
            Self::PerpetualTrackedAddresses(_) => 2 * 60 * SECONDS_PER_MINUTE,
            Self::PerpetualActiveAddresses(_) => 30 * SECONDS_PER_MINUTE,
            Self::PerpetualPriorityAddresses(_) => 30 * SECONDS_PER_MINUTE,
            Self::PerpetualObserverCheckpoint(_, _) => 30 * SECONDS_PER_DAY,
            Self::PendingTransactions(_) => 30 * SECONDS_PER_DAY,
            Self::TransactionFeeEstimates(_) => 5 * SECONDS_PER_YEAR,
            Self::TransactionFeeEstimatesFresh(_) => SECONDS_PER_HOUR,
            Self::ScanSafe(_, _, ttl) => *ttl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CacheKey, SECONDS_PER_DAY};

    #[test]
    fn test_fetch_transaction() {
        let key = CacheKey::FetchTransaction("ethereum", "0x123");
        assert_eq!(key.key(), "fetch:transaction:ethereum:0x123");
        assert_eq!(key.ttl(), 30 * SECONDS_PER_DAY);
    }

    #[test]
    fn test_scan_safe() {
        let key = CacheKey::ScanSafe("website", "example.com", 3600);
        assert_eq!(key.key(), "scan:safe:website:example.com");
        assert_eq!(key.ttl(), 3600);
    }

    #[test]
    fn test_fiat_quote() {
        let key = CacheKey::FiatQuote(1, 2, "quote");
        assert_eq!(key.key(), "fiat:quote:1:2:quote");
        assert_eq!(key.ttl(), 15 * 60);
    }

    #[test]
    fn test_ip_checks_last_one_day() {
        assert_eq!(CacheKey::ReferralIpCheck("1.1.1.1").ttl(), SECONDS_PER_DAY);
        assert_eq!(CacheKey::FiatIpCheck("1.1.1.1").ttl(), SECONDS_PER_DAY);
    }
}
