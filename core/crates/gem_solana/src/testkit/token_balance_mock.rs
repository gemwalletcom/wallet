use num_bigint::BigUint;

use crate::models::{TokenAmount, TokenBalance};

impl TokenBalance {
    pub fn mock(mint: &str, owner: &str, amount: u64) -> Self {
        Self {
            account_index: 0,
            mint: mint.to_string(),
            owner: owner.to_string(),
            ui_token_amount: TokenAmount { amount: BigUint::from(amount), decimals: 6 },
        }
    }
}
