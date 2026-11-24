use crate::{Delegation, DelegationValidator};

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum EarnType {
    Deposit(DelegationValidator),
    Withdraw(Delegation),
}

impl EarnType {
    pub fn provider(&self) -> &DelegationValidator {
        match self {
            EarnType::Deposit(provider) => provider,
            EarnType::Withdraw(delegation) => &delegation.validator,
        }
    }

    pub fn provider_id(&self) -> &str {
        &self.provider().id
    }
}
