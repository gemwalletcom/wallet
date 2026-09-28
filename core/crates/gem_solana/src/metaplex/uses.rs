use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Uses {
    pub use_method: UseMethod,
    pub remaining: u64,
    pub total: u64,
}
