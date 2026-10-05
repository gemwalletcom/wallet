use std::error::Error;
use std::fmt;

use primitives::Localize;
use rewards::{ReferralError, RewardsError, RewardsRedemptionError};
use storage::DatabaseError;

#[derive(Debug)]
pub enum RewardsServiceError {
    Rejected(RewardsError),
    Storage(DatabaseError),
    Internal(Box<dyn Error + Send + Sync>),
}

impl RewardsServiceError {
    pub(crate) fn referral(error: ReferralError, locale: &str) -> Self {
        Self::Rejected(RewardsError::Referral(error.localize(locale)))
    }

    pub(crate) fn redemption(error: RewardsRedemptionError, locale: &str) -> Self {
        Self::Rejected(RewardsError::Redemption(error.localize(locale)))
    }
}

impl fmt::Display for RewardsServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rejected(error) => write!(f, "{error}"),
            Self::Storage(error) => write!(f, "{error}"),
            Self::Internal(error) => write!(f, "{error}"),
        }
    }
}

impl Error for RewardsServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Rejected(_) => None,
            Self::Storage(error) => Some(error),
            Self::Internal(error) => Some(error.as_ref()),
        }
    }
}

impl From<RewardsError> for RewardsServiceError {
    fn from(error: RewardsError) -> Self {
        Self::Rejected(error)
    }
}

impl From<DatabaseError> for RewardsServiceError {
    fn from(error: DatabaseError) -> Self {
        Self::Storage(error)
    }
}

impl From<Box<dyn Error + Send + Sync>> for RewardsServiceError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        Self::Internal(error)
    }
}
