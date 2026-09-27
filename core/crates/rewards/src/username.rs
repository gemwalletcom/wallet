use crate::error::UsernameValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsernameRules {
    pub min_length: usize,
    pub max_length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardIdentity {
    pub username: String,
    pub wallet_address: String,
}

impl RewardIdentity {
    pub fn is_custom(&self, rules: &UsernameRules) -> bool {
        !self.username.eq_ignore_ascii_case(&self.wallet_address) && validate_username(&self.username, rules).is_ok()
    }

    pub fn referral_code(&self, rules: &UsernameRules) -> Option<String> {
        self.is_custom(rules).then(|| self.username.clone())
    }
}

pub fn validate_username(username: &str, rules: &UsernameRules) -> Result<(), UsernameValidationError> {
    let length = username.len();
    if length < rules.min_length {
        return Err(UsernameValidationError::TooShort(rules.min_length));
    }
    if length > rules.max_length {
        return Err(UsernameValidationError::TooLong(rules.max_length));
    }
    if !username.chars().all(|character| character.is_ascii_alphanumeric()) {
        return Err(UsernameValidationError::InvalidCharacters);
    }
    Ok(())
}

pub fn validate_username_available(is_taken: bool) -> Result<(), UsernameValidationError> {
    if is_taken { Err(UsernameValidationError::AlreadyTaken) } else { Ok(()) }
}

pub fn validate_wallet_without_username(identity: &RewardIdentity, rules: &UsernameRules) -> Result<(), UsernameValidationError> {
    if identity.is_custom(rules) {
        return Err(UsernameValidationError::WalletHasUsername);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        let rules = UsernameRules::mock();
        assert_eq!(validate_username("abcd", &rules), Ok(()));
        assert_eq!(validate_username("user123", &rules), Ok(()));
        assert_eq!(validate_username("1234567890123456", &rules), Ok(()));
        assert_eq!(validate_username("abc", &rules), Err(UsernameValidationError::TooShort(4)));
        assert_eq!(validate_username("12345678901234567", &rules), Err(UsernameValidationError::TooLong(16)));
        for username in ["user_name", "user-name", "user.name", "user name"] {
            assert_eq!(validate_username(username, &rules), Err(UsernameValidationError::InvalidCharacters));
        }

        let rules = UsernameRules { min_length: 2, max_length: 20 };
        assert_eq!(validate_username("ab", &rules), Ok(()));
        assert_eq!(validate_username("12345678901234567", &rules), Ok(()));
        assert_eq!(validate_username("a", &rules), Err(UsernameValidationError::TooShort(2)));
        assert_eq!(validate_username("123456789012345678901", &rules), Err(UsernameValidationError::TooLong(20)));
    }

    #[test]
    fn test_validate_username_available() {
        assert_eq!(validate_username_available(false), Ok(()));
        assert_eq!(validate_username_available(true), Err(UsernameValidationError::AlreadyTaken));
    }

    #[test]
    fn test_validate_wallet_without_username() {
        let rules = UsernameRules::mock();
        assert_eq!(validate_wallet_without_username(&RewardIdentity::mock_default(), &rules), Ok(()));
        assert_eq!(validate_wallet_without_username(&RewardIdentity::mock(), &rules), Err(UsernameValidationError::WalletHasUsername));
    }

    #[test]
    fn test_reward_identity() {
        let rules = UsernameRules::mock();
        assert!(RewardIdentity::mock().is_custom(&rules));
        assert_eq!(RewardIdentity::mock().referral_code(&rules), Some("alice".to_string()));
        assert!(!RewardIdentity::mock_default().is_custom(&rules));
        assert_eq!(RewardIdentity::mock_default().referral_code(&rules), None);

        let legacy = RewardIdentity {
            username: "wallet_1".to_string(),
            ..RewardIdentity::mock()
        };
        assert_eq!(legacy.referral_code(&rules), None);

        let default_identity = RewardIdentity::mock_default();
        let recased = RewardIdentity {
            username: default_identity.wallet_address.to_uppercase(),
            ..default_identity.clone()
        };
        let long_rules = UsernameRules { min_length: 4, max_length: 64 };
        assert!(!recased.is_custom(&long_rules));
        assert!(!default_identity.is_custom(&long_rules));

        let long_custom = RewardIdentity {
            username: "a".repeat(50),
            ..RewardIdentity::mock()
        };
        assert!(!long_custom.is_custom(&rules));
        assert!(long_custom.is_custom(&long_rules));
    }
}
