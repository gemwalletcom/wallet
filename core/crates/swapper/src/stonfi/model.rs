use serde::{Deserialize, Serialize};

use super::constants::is_supported_v2;
use crate::Route;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwapSimulation {
    pub offer_jetton_wallet: String,
    pub ask_jetton_wallet: String,
    pub router: Router,
    pub ask_units: String,
    pub min_ask_units: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Router {
    pub address: String,
    pub major_version: u8,
    pub minor_version: u8,
}

impl Router {
    pub(super) fn is_supported_v2(&self) -> bool {
        is_supported_v2(self.major_version, self.minor_version)
    }
}

#[derive(Debug)]
pub(super) struct QuotePath {
    pub(super) to_value: String,
    pub(super) routes: Vec<Route>,
}
