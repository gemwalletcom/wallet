use alloy_primitives::{Address, address, keccak256};
use alloy_sol_types::SolValue;
use primitives::Chain;

use super::v2::{clone_address, sort_tokens};
use crate::uniswap::path::TokenPair;

pub const METAQUOTER: &str = "0x1dDfeC5058180C207F55BB77D95AcDd0d8227868";
pub const METAROUTER: &str = "0x0fEf9e88A7b76BCDBF451780C6774FE32965Ee98";
const POOL_IMPLEMENTATION: Address = address!("0x1642634C8CB1C9f1B84c9a1a08534F8c3F4E89f8");

pub fn get_factory_by_chain(chain: &Chain) -> Option<Address> {
    match chain {
        Chain::Base => Some(address!("0x9CB220581b91b0C83223012FE69688b92aD07a76")),
        Chain::Robinhood => Some(address!("0xc63d7338c178e805c544B8f897066bD3BBcBE136")),
        Chain::Arc => Some(address!("0x7F83235643181E440433400a027618Cd50879751")),
        _ => None,
    }
}

pub fn pool_address(factory: &Address, pair: &TokenPair) -> Address {
    let (token0, token1) = sort_tokens(&pair.token_in, &pair.token_out);
    let salt = keccak256((*token0, *token1, pair.fee_tier.as_tick_spacing()).abi_encode());
    clone_address(factory, &POOL_IMPLEMENTATION, salt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uniswap::FeeTier;
    use primitives::asset_constants::{ARC_EURC_TOKEN_ID, ARC_USDC_TOKEN_ID, BASE_CBBTC_TOKEN_ID, BASE_USDC_TOKEN_ID, BASE_WETH_TOKEN_ID, ROBINHOOD_USDG_TOKEN_ID, ROBINHOOD_WETH_TOKEN_ID};

    #[test]
    fn test_pool_address() {
        let cases = [
            (Chain::Base, BASE_CBBTC_TOKEN_ID, BASE_USDC_TOKEN_ID, FeeTier::Hundred, address!("0x784e539f15ca2d19449a6311c5dab6cf37d8f1ba")),
            (Chain::Base, BASE_USDC_TOKEN_ID, BASE_CBBTC_TOKEN_ID, FeeTier::Hundred, address!("0x784e539f15ca2d19449a6311c5dab6cf37d8f1ba")),
            (Chain::Base, BASE_WETH_TOKEN_ID, BASE_USDC_TOKEN_ID, FeeTier::Fifty, address!("0x4a7af46bc8abc00048c6c7920a1488daf568366c")),
            (Chain::Robinhood, ROBINHOOD_USDG_TOKEN_ID, ROBINHOOD_WETH_TOKEN_ID, FeeTier::Fifty, address!("0x7615926023bb149f36095645c2a81dc367d39750")),
            (Chain::Arc, ARC_EURC_TOKEN_ID, ARC_USDC_TOKEN_ID, FeeTier::One, address!("0x6f57b4c22155cef09f17184c954f227ff0493742")),
        ];

        for (chain, token_in, token_out, fee_tier, pool) in cases {
            let pair = TokenPair {
                token_in: token_in.parse().unwrap(),
                token_out: token_out.parse().unwrap(),
                fee_tier,
            };
            assert_eq!(pool_address(&get_factory_by_chain(&chain).unwrap(), &pair), pool);
        }
    }
}
