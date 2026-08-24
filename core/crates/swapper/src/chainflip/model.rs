use serde::{Deserialize, Serialize};

use super::broker::DcaParameters;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ChainflipRouteData {
    pub boost_fee: Option<u32>,
    pub estimated_price: f64,
    pub dca_parameters: Option<DcaParameters>,
}
