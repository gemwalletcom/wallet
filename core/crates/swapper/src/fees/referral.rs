use super::DEFAULT_SWAP_FEE_BPS;
use primitives::{
    Chain, ChainType,
    swap::{
        APTOS_REFERRAL_ADDRESS, COSMOS_REFERRAL_ADDRESS, EVM_REFERRAL_ADDRESS, INJECTIVE_REFERRAL_ADDRESS, NEAR_REFERRAL_ADDRESS, SOLANA_REFERRAL_ADDRESS, SUI_REFERRAL_ADDRESS, THORCHAIN_REFERRAL_ADDRESS, TON_REFERRAL_ADDRESS,
        TRON_REFERRAL_ADDRESS,
    },
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ReferralFees {
    pub evm: ReferralFee,
    pub solana: ReferralFee,
    pub thorchain: ReferralFee,
    pub sui: ReferralFee,
    pub ton: ReferralFee,
    pub tron: ReferralFee,
    pub near: ReferralFee,
    pub aptos: ReferralFee,
    pub cosmos: ReferralFee,
    pub injective: ReferralFee,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ReferralFee {
    pub address: String,
    pub bps: u32,
}

impl ReferralFees {
    pub fn for_chain(&self, chain: Chain) -> Option<&ReferralFee> {
        let fee = match chain.chain_type() {
            ChainType::Ethereum => &self.evm,
            ChainType::Solana => &self.solana,
            ChainType::Sui => &self.sui,
            ChainType::Ton => &self.ton,
            ChainType::Tron => &self.tron,
            ChainType::Near => &self.near,
            ChainType::Aptos => &self.aptos,
            ChainType::Cosmos => match chain {
                Chain::Thorchain => &self.thorchain,
                Chain::Injective => &self.injective,
                _ => &self.cosmos,
            },
            ChainType::Bitcoin | ChainType::Xrp | ChainType::Stellar | ChainType::Algorand | ChainType::Polkadot | ChainType::Cardano | ChainType::HyperCore => return None,
        };
        Some(fee)
    }

    pub fn bps_for_chain(&self, chain: Chain) -> u32 {
        self.for_chain(chain).map(|fee| fee.bps).unwrap_or(0)
    }
}

pub fn default_referral_fees() -> ReferralFees {
    ReferralFees {
        evm: ReferralFee {
            address: EVM_REFERRAL_ADDRESS.into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
        solana: ReferralFee {
            address: SOLANA_REFERRAL_ADDRESS.into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
        thorchain: ReferralFee {
            address: THORCHAIN_REFERRAL_ADDRESS.into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
        sui: ReferralFee {
            address: SUI_REFERRAL_ADDRESS.into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
        ton: ReferralFee {
            address: TON_REFERRAL_ADDRESS.into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
        tron: ReferralFee {
            address: TRON_REFERRAL_ADDRESS.into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
        near: ReferralFee {
            address: NEAR_REFERRAL_ADDRESS.into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
        aptos: ReferralFee {
            address: APTOS_REFERRAL_ADDRESS.into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
        cosmos: ReferralFee {
            address: COSMOS_REFERRAL_ADDRESS.into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
        injective: ReferralFee {
            address: INJECTIVE_REFERRAL_ADDRESS.into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
    }
}

fn default_referral_fee(chain: Chain) -> ReferralFee {
    default_referral_fees().for_chain(chain).cloned().unwrap_or_default()
}

pub fn default_referral_address(chain: Chain) -> String {
    default_referral_fee(chain).address
}
