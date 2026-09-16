pub mod client;
pub mod mapper;
pub mod model;
mod target;
#[cfg(any(test, feature = "testkit"))]
pub mod testkit;

pub use self::mapper::{get_chain_for_coinmarketcap_platform, get_coinmarketcap_logo_url};
pub use self::model::*;

use gem_client::ReqwestClient;
pub type CoinMarketCapClient = client::CoinMarketCapClient<ReqwestClient>;
