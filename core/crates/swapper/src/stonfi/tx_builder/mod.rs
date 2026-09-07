mod message;
mod model;
mod v1;
mod v2;

use super::model::Router;
use crate::SwapperError;
use num_bigint::BigUint;

pub use model::{NextSwapParams, ReferralParams, SwapTransactionParams, TxParams};

#[derive(Debug, Clone, Copy)]
enum RouterVersion {
    V1,
    V2,
}

pub fn native_attachment() -> BigUint {
    BigUint::from(v2::TON_TO_JETTON_ATTACHMENT)
}

pub fn build_swap_transaction(params: SwapTransactionParams<'_>) -> Result<TxParams, SwapperError> {
    match (router_version(&params.simulation.router)?, params.next_swap.is_some()) {
        (RouterVersion::V1, false) => v1::build_swap_transaction(params),
        (RouterVersion::V1, true) => Err(SwapperError::ComputeQuoteError("STON.fi v1 multi-hop swap is not supported".into())),
        (RouterVersion::V2, _) => v2::build_swap_transaction(params),
    }
}

fn router_version(router: &Router) -> Result<RouterVersion, SwapperError> {
    match router.major_version {
        1 => Ok(RouterVersion::V1),
        2 => match router.minor_version {
            1 | 2 => Ok(RouterVersion::V2),
            minor => Err(SwapperError::ComputeQuoteError(format!("Unsupported STON.fi v2 router minor version: {minor}"))),
        },
        major => Err(SwapperError::ComputeQuoteError(format!("Unsupported STON.fi router major version: {major}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stonfi::model::SwapSimulation;
    use std::str::FromStr;

    #[test]
    fn test_native_attachment_covers_the_ton_to_jetton_message_of_every_router_version() {
        for simulation in [include_str!("../testdata/v1_simulation.json"), include_str!("../testdata/v2_simulation.json")] {
            let simulation: SwapSimulation = serde_json::from_str(simulation).unwrap();
            let params = SwapTransactionParams::mock(&simulation);
            let from_value = BigUint::from_str(params.from_value).unwrap();

            let transaction = build_swap_transaction(params).unwrap();

            let attached = BigUint::from_str(&transaction.value).unwrap() - from_value;
            assert!(attached <= native_attachment(), "router v{} attaches {attached}", simulation.router.major_version);
        }
    }
}
