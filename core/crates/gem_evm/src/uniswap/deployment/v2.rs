use alloy_primitives::{Address, B256, U256, b256, hex, keccak256};

use crate::uniswap::FeeTier;

const UNISWAP_V2_PAIR_INIT_CODE_HASH: B256 = b256!("0x96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f");
const PANCAKESWAP_V2_PAIR_INIT_CODE_HASH: B256 = b256!("0x00fb7f630766e6a796048ea87d01acd3068e8ff67d078148a3fa3f4a84f69bd5");
const FEE_TIER_DENOMINATOR: u32 = 1_000_000;
const CLONE_CREATION_PREFIX: [u8; 20] = hex!("3d602d80600a3d3981f3363d3d373d3d3d363d73");
const CLONE_CREATION_SUFFIX: [u8; 15] = hex!("5af43d82803e903d91602b57fd5bf3");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SolidlyDeployment {
    pub factory: Address,
    pub implementation: Address,
    pub router: Address,
}

impl SolidlyDeployment {
    pub fn pool_address(&self, token_a: &Address, token_b: &Address, stable: bool) -> Address {
        let (token0, token1) = sort_tokens(token_a, token_b);
        let salt = keccak256([token0.as_slice(), token1.as_slice(), &[u8::from(stable)]].concat());
        clone_address(&self.factory, &self.implementation, salt)
    }
}

pub(super) fn clone_address(factory: &Address, implementation: &Address, salt: B256) -> Address {
    let creation_code = [CLONE_CREATION_PREFIX.as_slice(), implementation.as_slice(), CLONE_CREATION_SUFFIX.as_slice()].concat();
    factory.create2(salt, keccak256(creation_code))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct V2Deployment {
    pub factory: Address,
    pub init_code_hash: B256,
    pub fee_tier: FeeTier,
}

impl V2Deployment {
    pub fn uniswap(factory: Address) -> Self {
        Self {
            factory,
            init_code_hash: UNISWAP_V2_PAIR_INIT_CODE_HASH,
            fee_tier: FeeTier::ThreeThousand,
        }
    }

    pub fn pancakeswap(factory: Address) -> Self {
        Self {
            factory,
            init_code_hash: PANCAKESWAP_V2_PAIR_INIT_CODE_HASH,
            fee_tier: FeeTier::TwoThousandFiveHundred,
        }
    }

    pub fn pair_address(&self, token_a: &Address, token_b: &Address) -> Address {
        let (token0, token1) = sort_tokens(token_a, token_b);
        let salt = keccak256([token0.as_slice(), token1.as_slice()].concat());
        self.factory.create2(salt, self.init_code_hash)
    }

    pub fn amount_out(&self, amount_in: U256, reserve_in: U256, reserve_out: U256) -> Option<U256> {
        constant_product_amount_out(self.fee_tier as u32, amount_in, reserve_in, reserve_out)
    }
}

pub fn round_trip_floor(fee: u32, amount_in: U256) -> Option<U256> {
    let kept = U256::from(FEE_TIER_DENOMINATOR.checked_sub(fee)?);
    Some(amount_in.checked_mul(kept)?.checked_mul(kept)? / U256::from(FEE_TIER_DENOMINATOR).pow(U256::from(2)))
}

pub fn constant_product_amount_out(fee: u32, amount_in: U256, reserve_in: U256, reserve_out: U256) -> Option<U256> {
    if reserve_in.is_zero() || reserve_out.is_zero() {
        return None;
    }
    let amount_in_with_fee = amount_in.checked_mul(U256::from(FEE_TIER_DENOMINATOR.checked_sub(fee)?))?;
    let numerator = amount_in_with_fee.checked_mul(reserve_out)?;
    let denominator = reserve_in.checked_mul(U256::from(FEE_TIER_DENOMINATOR))?.checked_add(amount_in_with_fee)?;
    Some(numerator / denominator)
}

pub fn sort_tokens<'a>(token_a: &'a Address, token_b: &'a Address) -> (&'a Address, &'a Address) {
    if token_a < token_b { (token_a, token_b) } else { (token_b, token_a) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;
    use primitives::asset_constants::{ETHEREUM_USDC_TOKEN_ID, ETHEREUM_WETH_TOKEN_ID, ROBINHOOD_WETH_TOKEN_ID};

    #[test]
    fn test_pair_address() {
        let ethereum = V2Deployment::uniswap(address!("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"));
        let robinhood = V2Deployment::uniswap(address!("0x8bcEaA40B9AcdfAedF85AdF4FF01F5Ad6517937f"));
        let smartchain = V2Deployment::pancakeswap(address!("0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73"));
        let usdc: Address = ETHEREUM_USDC_TOKEN_ID.parse().unwrap();
        let weth: Address = ETHEREUM_WETH_TOKEN_ID.parse().unwrap();
        let robinhood_weth: Address = ROBINHOOD_WETH_TOKEN_ID.parse().unwrap();
        let arrow = address!("0xf2915d1e3C1B0c769d0c756Ec43F1c1f6c99cD03");
        let wbnb = address!("0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c");
        let babydoge = address!("0xc748673057861a797275CD8A068AbB95A902e8de");

        assert_eq!(ethereum.pair_address(&usdc, &weth), address!("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"));
        assert_eq!(ethereum.pair_address(&weth, &usdc), address!("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"));
        assert_eq!(robinhood.pair_address(&robinhood_weth, &arrow), address!("0xE40d98D88038e0B844f844dce6Ae3c79ec01ec53"));
        assert_eq!(smartchain.pair_address(&babydoge, &wbnb), address!("0xc736cA3d9b1E90Af4230BD8F9626528B3D4e0Ee0"));
    }

    #[test]
    fn test_solidly_pool_address() {
        let aerodrome = SolidlyDeployment {
            factory: address!("0x420DD381b31aEf6683db6B902084cB0FFECe40Da"),
            implementation: address!("0xA4e46b4f701c62e14DF11B48dCe76A7d793CD6d7"),
            router: Address::ZERO,
        };
        let weth = address!("0x4200000000000000000000000000000000000006");
        let usdc = address!("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");
        let usdbc = address!("0xd9aAEc86B65D86f6A7B5B1b0c42FFA531710b6CA");

        assert_eq!(aerodrome.pool_address(&weth, &usdc, false), address!("0xcDAC0d6c6C59727a65F871236188350531885C43"));
        assert_eq!(aerodrome.pool_address(&usdc, &weth, false), address!("0xcDAC0d6c6C59727a65F871236188350531885C43"));
        assert_eq!(aerodrome.pool_address(&weth, &usdc, true), address!("0x3548029694fbB241D45FB24Ba0cd9c9d4E745f16"));
        assert_eq!(aerodrome.pool_address(&usdc, &usdbc, true), address!("0x27a8Afa3Bd49406e48a074350fB7b2020c43B2bD"));
    }

    #[test]
    fn test_amount_out() {
        let uniswap = V2Deployment::uniswap(Address::ZERO);
        let pancakeswap = V2Deployment::pancakeswap(Address::ZERO);
        let reserve_in = U256::from(5_910_905_346_459_282_145_000u128);
        let reserve_out = U256::from(11_977_652_183_942_310_442_955_811u128);
        let amount_in = U256::from(10_000_000_000_000_000u64);

        assert_eq!(uniswap.amount_out(amount_in, reserve_in, reserve_out), Some(U256::from(20_202_825_769_027_823_094u128)));
        assert_eq!(pancakeswap.amount_out(amount_in, reserve_in, reserve_out), Some(U256::from(20_212_957_560_239_272_486u128)));
        assert_eq!(uniswap.amount_out(amount_in, U256::ZERO, reserve_out), None);
        assert_eq!(uniswap.amount_out(U256::MAX, reserve_in, reserve_out), None);
    }
}
