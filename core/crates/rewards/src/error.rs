use std::error::Error;
use std::fmt;

use config_keys::RateLimitKey;
use localizer::LanguageLocalizer;
use primitives::Localize;

#[derive(Debug)]
pub enum RewardsError {
    Username(String),
    Referral(String),
}

impl fmt::Display for RewardsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RewardsError::Username(msg) => write!(f, "{}", msg),
            RewardsError::Referral(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for RewardsError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferralValidationError {
    CodeDoesNotExist,
    DeviceAlreadyUsed,
    CannotReferSelf,
    EligibilityExpired(i64),
    RewardsNotEnabled(String),
}

impl fmt::Display for ReferralValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CodeDoesNotExist => write!(f, "Referral code does not exist"),
            Self::DeviceAlreadyUsed => write!(f, "This device has already been used to apply a referral code"),
            Self::CannotReferSelf => write!(f, "Cannot use your own referral code"),
            Self::EligibilityExpired(days) => write!(f, "eligibility_expired: {} days", days),
            Self::RewardsNotEnabled(user) => write!(f, "Rewards are not enabled for {}", user),
        }
    }
}

impl Error for ReferralValidationError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferralConfirmationError {
    AlreadyVerified,
    CodeMismatch,
    DeviceMismatch,
}

impl fmt::Display for ReferralConfirmationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyVerified => write!(f, "Referral already verified"),
            Self::CodeMismatch => write!(f, "Referral code does not match pending referral"),
            Self::DeviceMismatch => write!(f, "Must verify from same device"),
        }
    }
}

impl Error for ReferralConfirmationError {}

#[derive(Debug, Clone, PartialEq)]
pub enum UsernameValidationError {
    Invalid(String),
    AlreadyTaken,
}

impl fmt::Display for UsernameValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(msg) => write!(f, "{}", msg),
            Self::AlreadyTaken => write!(f, "Username already taken"),
        }
    }
}

impl Error for UsernameValidationError {}

#[derive(Debug)]
pub enum ReferralError {
    Validation(ReferralValidationError),
    Confirmation(ReferralConfirmationError),
    ReferrerLimitReached,
    RiskScoreExceeded { score: i64, max_allowed: i64 },
    DuplicateAttempt,
    IpTorNotAllowed,
    IpCountryIneligible(String),
    LimitReached,
    InvalidDeviceToken(String),
    Internal(String),
}

impl fmt::Display for ReferralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReferralError::Validation(e) => write!(f, "{}", e),
            ReferralError::Confirmation(e) => write!(f, "{}", e),
            ReferralError::ReferrerLimitReached => write!(f, "referrer_limit_reached"),
            ReferralError::RiskScoreExceeded { score, max_allowed } => write!(f, "risk_score: {} (max allowed: {})", score, max_allowed),
            ReferralError::DuplicateAttempt => write!(f, "duplicate_attempt"),
            ReferralError::IpTorNotAllowed => write!(f, "ip_tor_not_allowed"),
            ReferralError::IpCountryIneligible(country) => write!(f, "ip_country_ineligible: {}", country),
            ReferralError::LimitReached => write!(f, "limit_reached"),
            ReferralError::InvalidDeviceToken(reason) => write!(f, "invalid_device_token: {}", reason),
            ReferralError::Internal(message) => write!(f, "{}", message),
        }
    }
}

impl Error for ReferralError {}

impl Localize for ReferralError {
    fn localize(&self, locale: &str) -> String {
        let localizer = LanguageLocalizer::new_with_language(locale);
        match self {
            Self::Validation(ReferralValidationError::CodeDoesNotExist) => localizer.rewards_error_referral_code_not_exist(),
            Self::Validation(ReferralValidationError::DeviceAlreadyUsed) => localizer.rewards_error_referral_device_already_used(),
            Self::Validation(ReferralValidationError::CannotReferSelf) => localizer.rewards_error_referral_cannot_refer_self(),
            Self::Validation(ReferralValidationError::EligibilityExpired(days)) => localizer.rewards_error_referral_eligibility_expired(*days),
            Self::Validation(ReferralValidationError::RewardsNotEnabled(_)) => localizer.rewards_error_referral_rewards_not_enabled(),
            Self::Confirmation(ReferralConfirmationError::AlreadyVerified) => localizer.rewards_error_referral_device_already_used(),
            Self::Confirmation(ReferralConfirmationError::CodeMismatch | ReferralConfirmationError::DeviceMismatch) => localizer.rewards_error_referral_limit_reached(),
            Self::ReferrerLimitReached => localizer.rewards_error_referral_referrer_limit_reached(),
            Self::IpCountryIneligible(country) => localizer.rewards_error_referral_country_ineligible(country),
            Self::RiskScoreExceeded { .. } | Self::DuplicateAttempt | Self::IpTorNotAllowed | Self::LimitReached | Self::InvalidDeviceToken(_) => localizer.rewards_error_referral_limit_reached(),
            Self::Internal(_) => localizer.errors_generic(),
        }
    }
}

impl From<ReferralConfirmationError> for ReferralError {
    fn from(error: ReferralConfirmationError) -> Self {
        ReferralError::Confirmation(error)
    }
}

impl From<ReferralValidationError> for ReferralError {
    fn from(error: ReferralValidationError) -> Self {
        ReferralError::Validation(error)
    }
}

impl ReferralError {
    pub fn internal(error: impl fmt::Display) -> Self {
        Self::Internal(error.to_string())
    }
}

impl From<Box<dyn Error + Send + Sync>> for ReferralError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        Self::internal(error)
    }
}

#[derive(Debug)]
pub enum RewardsRedemptionError {
    NotEligible(String),
    LimitReached,
    AccountTooNew,
    CooldownNotElapsed,
    NotEnoughPoints,
    OptionNotAvailable,
    NoUsername,
}

impl fmt::Display for RewardsRedemptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RewardsRedemptionError::NotEligible(msg) => write!(f, "{}", msg),
            RewardsRedemptionError::LimitReached => write!(f, "Redemption limit reached"),
            RewardsRedemptionError::AccountTooNew => write!(f, "Account too new for redemption"),
            RewardsRedemptionError::CooldownNotElapsed => write!(f, "Must wait after recent referral activity"),
            RewardsRedemptionError::NotEnoughPoints => write!(f, "Not enough points"),
            RewardsRedemptionError::OptionNotAvailable => write!(f, "Redemption option is no longer available"),
            RewardsRedemptionError::NoUsername => write!(f, "No username found for address"),
        }
    }
}

impl Error for RewardsRedemptionError {}

#[derive(Debug)]
pub enum UsernameError {
    LimitReached(RateLimitKey),
    Validation(UsernameValidationError),
    Internal(String),
}

impl fmt::Display for UsernameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UsernameError::LimitReached(key) => write!(f, "Username creation limit reached: {}", key.as_ref()),
            UsernameError::Validation(e) => write!(f, "{}", e),
            UsernameError::Internal(message) => write!(f, "{}", message),
        }
    }
}

impl Error for UsernameError {}

impl Localize for UsernameError {
    fn localize(&self, locale: &str) -> String {
        let localizer = LanguageLocalizer::new_with_language(locale);
        match self {
            Self::LimitReached(_) => localizer.rewards_error_username_daily_limit_reached(),
            Self::Validation(e) => e.to_string(),
            Self::Internal(_) => localizer.errors_generic(),
        }
    }
}

impl From<UsernameValidationError> for UsernameError {
    fn from(error: UsernameValidationError) -> Self {
        UsernameError::Validation(error)
    }
}

impl UsernameError {
    pub fn internal(error: impl fmt::Display) -> Self {
        Self::Internal(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use primitives::Localize;

    use super::*;

    #[test]
    fn test_internal_errors_localize_to_generic_text() {
        let raw = "duplicate key value violates unique constraint \"usernames_pkey\"";
        let generic = LanguageLocalizer::new_with_language("en").errors_generic();

        assert_eq!(UsernameError::internal(raw).localize("en"), generic);
        assert_eq!(ReferralError::Internal(raw.to_string()).localize("en"), generic);
        assert_eq!(UsernameError::internal(raw).to_string(), raw);
    }

    #[test]
    fn test_confirmation_errors_localize_to_referral_text() {
        let localizer = LanguageLocalizer::new_with_language("en");

        assert_eq!(ReferralError::from(ReferralConfirmationError::AlreadyVerified).localize("en"), localizer.rewards_error_referral_device_already_used());
        assert_eq!(ReferralError::from(ReferralConfirmationError::DeviceMismatch).localize("en"), localizer.rewards_error_referral_limit_reached());
    }
}
