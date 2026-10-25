mod client;
mod model;
mod provider;
mod target;

use primitives::Chain;

use crate::Deployment;

pub use provider::Squid;

pub const SQUID_COSMOS_MULTICALL: Deployment = Deployment::new(Chain::Osmosis, "osmo1n6ney9tsf55etz9nrmzyd8wa7e64qd3s06a74fqs30ka8pps6cvqtsycr6");
