use crate::models::{Info, Parsed, StakeDelegation, StakeInfo, TokenAccountData, TokenAccountInfo, TokenAccountInfoData};

impl TokenAccountInfo {
    pub fn mock_stake(activation_epoch: u64, deactivation_epoch: u64) -> Self {
        Self {
            pubkey: "stake1".to_string(),
            account: TokenAccountData {
                data: Parsed {
                    parsed: Info {
                        info: TokenAccountInfoData {
                            mint: None,
                            token_amount: None,
                            stake: Some(StakeInfo {
                                delegation: StakeDelegation {
                                    activation_epoch,
                                    deactivation_epoch,
                                    stake: "1000000".to_string(),
                                    voter: "validator1".to_string(),
                                },
                            }),
                        },
                    },
                },
                owner: "owner1".to_string(),
                lamports: 1000000,
            },
        }
    }
}
