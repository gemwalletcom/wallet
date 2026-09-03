mod constants;
mod contracts;
mod encode;
mod mapper;
mod models;

#[cfg(feature = "rpc")]
mod client;
#[cfg(feature = "rpc")]
mod parser;
#[cfg(feature = "rpc")]
mod staking;
#[cfg(feature = "rpc")]
mod target;

#[cfg(test)]
mod testkit;

#[cfg(feature = "rpc")]
pub use client::EverstakeClient;
#[cfg(feature = "rpc")]
pub use staking::EverstakeStakingClient;
