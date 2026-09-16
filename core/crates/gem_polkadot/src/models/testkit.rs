use crate::models::account::PolkadotAccountBalance;
use crate::models::rpc::{Extrinsic, ExtrinsicArguments, ExtrinsicInfo, ExtrinsicMethod};
use num_bigint::BigInt;

impl PolkadotAccountBalance {
    pub fn mock() -> Self {
        Self::mock_with_balances(31415926535, 0, 31415926535, 0)
    }

    pub fn mock_with_balances(free: u64, reserved: u64, frozen: u64, transferable: u64) -> Self {
        Self {
            free: BigInt::from(free),
            reserved: BigInt::from(reserved),
            frozen: BigInt::from(frozen),
            transferable: BigInt::from(transferable),
            nonce: 0,
        }
    }
}

impl Extrinsic {
    pub fn mock() -> Self {
        Self {
            hash: "hash123".to_string(),
            method: ExtrinsicMethod {
                pallet: String::new(),
                method: String::new(),
            },
            info: ExtrinsicInfo { partial_fee: None },
            success: true,
            args: ExtrinsicArguments::Other(serde_json::json!({})),
            signature: None,
        }
    }
}
