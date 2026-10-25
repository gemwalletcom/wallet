use super::{DEFAULT_SWAP_FEE_BPS, STABLECOIN_SWAP_FEE_BPS};
use crate::{
    QuoteRequest,
    fees::{bps_of, subtract_bps},
};
use gem_cosmos::address::CosmosAddress;
use gem_evm::slippage::BasisPointConvert;
use gem_solana::{USDC_TOKEN_MINT, USDS_TOKEN_MINT, USDT_TOKEN_MINT, WSOL_TOKEN_ADDRESS};
use primitives::chain_cosmos::CosmosChain;
use primitives::{
    AssetId, Chain, ChainType,
    swap::{
        APTOS_REFERRAL_ADDRESS, COSMOS_REFERRAL_ADDRESS, EVM_REFERRAL_ADDRESS, HUNDRED_PERCENT_IN_BPS, INJECTIVE_REFERRAL_ADDRESS, NEAR_REFERRAL_ADDRESS, SOLANA_REFERRAL_ADDRESS, STELLAR_REFERRAL_ADDRESS, SUI_REFERRAL_ADDRESS,
        THORCHAIN_REFERRAL_ADDRESS, TON_REFERRAL_ADDRESS, TRON_REFERRAL_ADDRESS,
    },
};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Rem, Sub};

pub const SOLANA_FEE_MINTS: [&str; 4] = [WSOL_TOKEN_ADDRESS, USDC_TOKEN_MINT, USDT_TOKEN_MINT, USDS_TOKEN_MINT];

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
    pub stellar: ReferralFee,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ReferralFee {
    pub address: String,
    pub bps: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeeSide {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReferralTransfer<T = u64> {
    pub side: FeeSide,
    pub value: T,
}

pub trait FeeAmount: BasisPointConvert + Mul<Output = Self> + Div<Output = Self> + Rem<Output = Self> + Add<Output = Self> + Sub<Output = Self> + PartialOrd {}

impl<T: BasisPointConvert + Mul<Output = T> + Div<Output = T> + Rem<Output = T> + Add<Output = T> + Sub<Output = T> + PartialOrd> FeeAmount for T {}

pub fn fee_side(fee_assets: &[AssetId], from_asset: &AssetId, to_asset: &AssetId) -> Option<FeeSide> {
    FeeSide::from_available(fee_assets.contains(from_asset), fee_assets.contains(to_asset))
}

fn saturating_sub<T: FeeAmount>(value: T, subtrahend: T) -> T {
    if value > subtrahend { value - subtrahend } else { T::from_u32(0) }
}

impl FeeSide {
    pub fn from_available(input: bool, output: bool) -> Option<Self> {
        match (input, output) {
            (true, _) => Some(Self::Input),
            (false, true) => Some(Self::Output),
            (false, false) => None,
        }
    }

    pub fn by_priority<T>(priority: &[T], is_input: impl Fn(&T) -> bool, is_output: impl Fn(&T) -> bool) -> Option<Self> {
        priority.iter().find_map(|preferred| Self::from_available(is_input(preferred), is_output(preferred)))
    }
}

impl ReferralFee {
    fn fee_value<T: FeeAmount>(&self, value: &T) -> T {
        bps_of(value, self.bps)
    }

    pub fn swap_value<T: FeeAmount>(&self, side: FeeSide, value: T) -> T {
        match side {
            FeeSide::Input => {
                let fee = self.fee_value(&value);
                saturating_sub(value, fee)
            }
            FeeSide::Output => value,
        }
    }

    pub fn value_for_swap<T: FeeAmount>(&self, side: FeeSide, swap_value: T) -> T {
        match side {
            FeeSide::Input => {
                let kept = HUNDRED_PERCENT_IN_BPS - self.bps;
                (swap_value * T::from_u32(HUNDRED_PERCENT_IN_BPS) + T::from_u32(kept - 1)) / T::from_u32(kept)
            }
            FeeSide::Output => swap_value,
        }
    }

    pub fn transfer<T: FeeAmount>(&self, side: FeeSide, value: T, output_value: T) -> ReferralTransfer<T> {
        let fee_base = match side {
            FeeSide::Input => value,
            FeeSide::Output => output_value,
        };
        ReferralTransfer { side, value: self.fee_value(&fee_base) }
    }
}

impl<T: FeeAmount> ReferralTransfer<T> {
    pub fn input_value(&self, swap_value: T) -> T {
        match self.side {
            FeeSide::Input => swap_value + self.value.clone(),
            FeeSide::Output => swap_value,
        }
    }

    pub fn received_value(&self, output_value: T) -> T {
        match self.side {
            FeeSide::Input => output_value,
            FeeSide::Output => saturating_sub(output_value, self.value.clone()),
        }
    }

    pub fn minimum_output(&self, output_value: T, slippage_bps: u32) -> T {
        let minimum_received = subtract_bps(&self.received_value(output_value), slippage_bps);
        match self.side {
            FeeSide::Input => minimum_received,
            FeeSide::Output => minimum_received + self.value.clone(),
        }
    }
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
            ChainType::Stellar => &self.stellar,
            ChainType::Cosmos => match chain {
                Chain::Thorchain => &self.thorchain,
                Chain::Injective => &self.injective,
                _ => &self.cosmos,
            },
            ChainType::Bitcoin | ChainType::Xrp | ChainType::Algorand | ChainType::Polkadot | ChainType::Cardano | ChainType::HyperCore => return None,
        };
        Some(fee)
    }
}

pub fn default_referral_fees() -> ReferralFees {
    referral_fees(DEFAULT_SWAP_FEE_BPS)
}

pub fn on_chain_referral_fees(request: &QuoteRequest) -> ReferralFees {
    match request.is_stablecoin_swap() {
        true => referral_fees(STABLECOIN_SWAP_FEE_BPS),
        false => default_referral_fees(),
    }
}

pub(crate) fn referral_fees(bps: u32) -> ReferralFees {
    let fee = |address: &str| ReferralFee { address: address.into(), bps };
    ReferralFees {
        evm: fee(EVM_REFERRAL_ADDRESS),
        solana: fee(SOLANA_REFERRAL_ADDRESS),
        thorchain: fee(THORCHAIN_REFERRAL_ADDRESS),
        sui: fee(SUI_REFERRAL_ADDRESS),
        ton: fee(TON_REFERRAL_ADDRESS),
        tron: fee(TRON_REFERRAL_ADDRESS),
        near: fee(NEAR_REFERRAL_ADDRESS),
        aptos: fee(APTOS_REFERRAL_ADDRESS),
        cosmos: fee(COSMOS_REFERRAL_ADDRESS),
        injective: fee(INJECTIVE_REFERRAL_ADDRESS),
        stellar: fee(STELLAR_REFERRAL_ADDRESS),
    }
}

fn default_referral_fee(chain: Chain) -> ReferralFee {
    default_referral_fees().for_chain(chain).cloned().unwrap_or_default()
}

pub fn default_referral_address(chain: Chain) -> String {
    let address = default_referral_fee(chain).address;
    match CosmosChain::from_chain(chain) {
        Some(cosmos_chain) if !matches!(chain, Chain::Cosmos | Chain::Thorchain | Chain::Injective) && !address.is_empty() => CosmosAddress::convert(&address, cosmos_chain.hrp()).unwrap_or_default(),
        _ => address,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SwapperQuoteAsset, testkit::mock_quote};
    use num_bigint::BigUint;
    use primitives::{asset_constants::*, swap::STELLAR_REFERRAL_FEE_ASSET_IDS};

    #[test]
    fn test_on_chain_referral_fees() {
        let asset = |chain: Chain, token_id: &str, symbol: &str| SwapperQuoteAsset::mock_with_asset_id(AssetId::from_token(chain, token_id), symbol, 6);
        let usdt = asset(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID, "USDT");
        let usdc = asset(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID, "USDC");
        let named_usdc = asset(Chain::Ethereum, "0x0000000000000000000000000000000000000001", "USDC");
        let eth = SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum));

        let bps = |from: &SwapperQuoteAsset, to: &SwapperQuoteAsset| on_chain_referral_fees(&mock_quote(from.clone(), to.clone())).evm.bps;

        assert_eq!(bps(&usdt, &usdc), STABLECOIN_SWAP_FEE_BPS);
        assert_eq!(bps(&usdc, &usdt), STABLECOIN_SWAP_FEE_BPS);
        assert_eq!(bps(&usdt, &eth), DEFAULT_SWAP_FEE_BPS);
        assert_eq!(bps(&named_usdc, &usdt), DEFAULT_SWAP_FEE_BPS);
    }

    #[test]
    fn test_fee_side_by_priority() {
        let priority = ["sol", "usdc"];
        let select = |input: &str, output: &str| FeeSide::by_priority(&priority, |preferred| *preferred == input, |preferred| *preferred == output);

        assert_eq!(select("sol", "usdc"), Some(FeeSide::Input));
        assert_eq!(select("usdc", "sol"), Some(FeeSide::Output));
        assert_eq!(select("bonk", "usdc"), Some(FeeSide::Output));
        assert_eq!(select("usdc", "bonk"), Some(FeeSide::Input));
        assert_eq!(select("bonk", "wif"), None);
    }

    #[test]
    fn test_fee_side_takes_the_fee_in_an_asset_the_fee_account_receives() {
        let stellar = |token_id: Option<&str>| AssetId::from(Chain::Stellar, token_id.map(str::to_string));
        let (xlm, usdc) = (stellar(None), STELLAR_USDC_ASSET_ID.clone());
        let eurc = stellar(Some("GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2::EURC"));
        let other_usdc = stellar(Some("GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2::USDC"));
        let side = |from: &AssetId, to: &AssetId| fee_side(&*STELLAR_REFERRAL_FEE_ASSET_IDS, from, to);

        assert_eq!(side(&xlm, &usdc), Some(FeeSide::Input));
        assert_eq!(side(&usdc, &xlm), Some(FeeSide::Input));
        assert_eq!(side(&usdc, &eurc), Some(FeeSide::Input));
        assert_eq!(side(&eurc, &xlm), Some(FeeSide::Output));
        assert_eq!(side(&eurc, &usdc), Some(FeeSide::Output));
        assert_eq!(side(&eurc, &other_usdc), None, "a USDC from another issuer is not one the fee account trusts");
    }

    #[test]
    fn test_fee_value() {
        let fee = ReferralFee { address: String::new(), bps: 50 };

        assert_eq!(fee.fee_value(&1_000_000u64), 5_000);
        assert_eq!(fee.fee_value(&199u64), 0);
        assert_eq!(fee.fee_value(&u64::MAX), u64::MAX / 200);
        assert_eq!(fee.fee_value(&BigUint::from(1_000_000u32)), BigUint::from(5_000u32));
        assert_eq!(ReferralFee { bps: 0, ..fee }.fee_value(&1_000_000u64), 0);
    }

    #[test]
    fn test_value_for_swap() {
        let fee = ReferralFee { address: String::new(), bps: 50 };

        assert_eq!(fee.value_for_swap(FeeSide::Input, 1_000_000_000u64), 1_005_025_126);
        assert_eq!(fee.swap_value(FeeSide::Input, 1_005_025_126u64), 1_000_000_001);
        assert_eq!(fee.value_for_swap(FeeSide::Output, 1_000_000_000u64), 1_000_000_000);
    }

    #[test]
    fn test_transfer() {
        let fee = ReferralFee { address: String::new(), bps: 50 };
        let input = fee.transfer(FeeSide::Input, 1_000_000u64, 2_000_000);
        let output = fee.transfer(FeeSide::Output, 1_000_000u64, 2_000_000);
        let output_big = fee.transfer(FeeSide::Output, BigUint::from(1_000_000u32), BigUint::from(2_000_000u32));

        assert_eq!(fee.swap_value(FeeSide::Input, 1_000_000u64), 995_000);
        assert_eq!(fee.swap_value(FeeSide::Output, 1_000_000u64), 1_000_000);
        assert_eq!(input, ReferralTransfer { side: FeeSide::Input, value: 5_000 });
        assert_eq!(output, ReferralTransfer { side: FeeSide::Output, value: 10_000 });
        assert_eq!(input.input_value(995_000), 1_000_000);
        assert_eq!(output.input_value(1_000_000), 1_000_000);
        assert_eq!(input.received_value(2_000_000), 2_000_000);
        assert_eq!(output.received_value(2_000_000), 1_990_000);
        assert_eq!(output_big.received_value(BigUint::from(2_000_000u32)), BigUint::from(1_990_000u32));
        assert_eq!(output.minimum_output(2_000_000, 100), 1_980_100, "1,990,000 kept less 1%, plus the fee paid out of it");
        assert_eq!(output_big.minimum_output(BigUint::from(2_000_000u32), 100), BigUint::from(1_980_100u32));
    }
}
