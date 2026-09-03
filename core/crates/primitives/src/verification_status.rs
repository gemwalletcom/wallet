use serde::{Deserialize, Serialize};
use typeshare::typeshare;

const SUSPICIOUS_MAX_RANK: i32 = 5;
const UNVERIFIED_MAX_RANK: i32 = 15;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Suspicious,
}

impl VerificationStatus {
    pub fn from_verified(is_verified: bool) -> Self {
        if is_verified { Self::Verified } else { Self::Unverified }
    }

    pub fn from_rank(rank: i32) -> Self {
        if rank <= SUSPICIOUS_MAX_RANK {
            Self::Suspicious
        } else if rank <= UNVERIFIED_MAX_RANK {
            Self::Unverified
        } else {
            Self::Verified
        }
    }

    pub fn is_verified(self) -> bool {
        match self {
            Self::Verified => true,
            Self::Unverified | Self::Suspicious => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_rank() {
        assert_eq!(VerificationStatus::from_rank(0), VerificationStatus::Suspicious);
        assert_eq!(VerificationStatus::from_rank(5), VerificationStatus::Suspicious);
        assert_eq!(VerificationStatus::from_rank(6), VerificationStatus::Unverified);
        assert_eq!(VerificationStatus::from_rank(15), VerificationStatus::Unverified);
        assert_eq!(VerificationStatus::from_rank(16), VerificationStatus::Verified);
    }
}
