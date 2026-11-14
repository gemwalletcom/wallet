mod asset;
mod chain;
pub mod client;
mod constants;
pub(crate) mod contracts;
pub mod memo;
pub mod model;
mod provider;
mod quote_data_mapper;
mod quote_mapper;
mod swap_mapper;
mod target;
#[cfg(test)]
mod testkit;

pub use provider::ThorChain;

use primitives::Chain;

use super::Deployment;

const QUOTE_MINIMUM: i64 = 0;
const QUOTE_INTERVAL: i64 = 1;
const QUOTE_QUANTITY: i64 = 0;
const DUST_THRESHOLD_MULTIPLIER: i64 = 2;

const DEPOSIT_GAS_LIMIT_BASE: u64 = 90_000;
const CALLDATA_GAS_PER_BYTE: u64 = 16;

const ROUTER_V6_1: &str = "0x00dc6100103BC402d490aEE3F9a5560cBd91f1d4";
pub const THORCHAIN_ROUTERS: &[Deployment] = &[
    Deployment::new(Chain::Ethereum, "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146"),
    Deployment::new(Chain::Ethereum, ROUTER_V6_1),
    Deployment::new(Chain::SmartChain, "0xb30eC53F98ff5947EDe720D32aC2da7e52A5f56b"),
    Deployment::new(Chain::SmartChain, ROUTER_V6_1),
    Deployment::new(Chain::AvalancheC, "0x8F66c4AE756BEbC49Ec8B81966DD8bba9f127549"),
    Deployment::new(Chain::AvalancheC, ROUTER_V6_1),
    Deployment::new(Chain::Base, "0x68208D99746b805a1Ae41421950A47b711E35681"),
    Deployment::new(Chain::Base, ROUTER_V6_1),
];

pub fn deposit_gas_limit(memo: &str) -> u64 {
    DEPOSIT_GAS_LIMIT_BASE + memo.len() as u64 * CALLDATA_GAS_PER_BYTE
}
