mod commands;
mod path;
mod quoter;

pub mod provider;
pub use provider::UniswapV4;

use primitives::Chain;

const DEFAULT_SWAP_GAS_LIMIT: u64 = 300_000;

fn default_swap_gas_limit(chain: &Chain) -> u64 {
    match chain {
        // 250k gas per new storage slot: first-time swaps overrun the standard default
        Chain::Tempo => 900_000,
        _ => DEFAULT_SWAP_GAS_LIMIT,
    }
}
