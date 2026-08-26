use primitives::stake_type::Resource;
use primitives::{Delegation, DelegationBase, DelegationState, DelegationValidator, Price, StakeChain, StakeProviderType};

pub type GemResource = Resource;
pub type GemDelegation = Delegation;
pub type GemDelegationBase = DelegationBase;
pub type GemDelegationValidator = DelegationValidator;
pub type GemDelegationState = DelegationState;
pub type GemStakeProviderType = StakeProviderType;
pub type GemPrice = Price;
pub type GemStakeChain = StakeChain;

#[uniffi::remote(Enum)]
pub enum GemStakeChain {
    Cosmos,
    Osmosis,
    Injective,
    Sei,
    Celestia,
    Ethereum,
    Solana,
    Sui,
    SmartChain,
    Monad,
    Tron,
    Aptos,
    HyperCore,
}
