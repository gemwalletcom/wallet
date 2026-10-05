mod config;
mod error;
mod ip_security_client;
mod redemption;
mod redemption_client;
pub(crate) mod repository;
mod rewards_abuse_checker;
mod rewards_client;
mod rewards_consumer;
mod rewards_eligibility_checker;
mod rewards_redemption_consumer;
mod risk;

#[cfg(test)]
pub(crate) use config::AbuseDetectionConfig;
pub use config::{ReferralVerificationConfig, username_rules};
pub use error::RewardsServiceError;
pub use ip_security_client::IpSecurityClient;
pub use redemption_client::RewardsRedemptionClient;
pub use rewards_abuse_checker::RewardsAbuseChecker;
pub use rewards_client::RewardsClient;
pub use rewards_consumer::RewardsConsumer;
pub use rewards_eligibility_checker::RewardsEligibilityChecker;
pub use rewards_redemption_consumer::{RedemptionRetryConfig, RewardsRedemptionConsumer};
