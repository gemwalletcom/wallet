mod commands;
mod path;
mod quoter;

pub mod provider;
pub use provider::UniswapV4;

use primitives::Chain;

const DEFAULT_SWAP_GAS_LIMIT: u64 = 300_000;

fn default_swap_gas_limit(chain: &Chain) -> u64 {
    match chain {
        Chain::Tempo => 900_000,
        _ => DEFAULT_SWAP_GAS_LIMIT,
    }
}
