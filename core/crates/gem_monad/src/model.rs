use num_bigint::BigUint;

use crate::contracts::IMonadStakingLens;

#[derive(Clone)]
pub struct LensDelegation {
    pub validator_id: u64,
    pub withdraw_id: u8,
    pub state: IMonadStakingLens::DelegationState,
    pub amount: BigUint,
    pub rewards: BigUint,
    pub completion_timestamp: u64,
}

impl LensDelegation {
    pub fn is_pending_withdrawal(&self) -> bool {
        match self.state {
            IMonadStakingLens::DelegationState::Deactivating | IMonadStakingLens::DelegationState::AwaitingWithdrawal => true,
            IMonadStakingLens::DelegationState::Active | IMonadStakingLens::DelegationState::Activating | IMonadStakingLens::DelegationState::__Invalid => false,
        }
    }
}

#[derive(Clone)]
pub struct LensValidator {
    pub validator_id: u64,
    pub commission: BigUint,
    pub apy_bps: u64,
    pub is_active: bool,
}

#[derive(Clone)]
pub struct LensBalance {
    pub staked: BigUint,
    pub pending: BigUint,
    pub rewards: BigUint,
}
