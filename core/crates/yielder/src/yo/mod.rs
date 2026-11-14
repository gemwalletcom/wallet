mod assets;
mod client;
mod contract;
mod mapper;
mod provider;

use assets::{YoAsset, supported_assets};
pub use provider::YoEarnProvider;

const YO_PARTNER_ID_GEM: u32 = 6548;
