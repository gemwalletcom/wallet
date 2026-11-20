#[cfg(feature = "rpc")]
mod constants;
#[cfg(feature = "rpc")]
mod contracts;
#[cfg(feature = "rpc")]
mod encode;
#[cfg(feature = "rpc")]
mod mapper;
#[cfg(feature = "rpc")]
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
