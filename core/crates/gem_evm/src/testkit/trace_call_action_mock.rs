#![cfg(feature = "rpc")]

use crate::rpc::model::TraceCallAction;
use crate::testkit::TEST_ADDRESS;
use primitives::hex;

impl TraceCallAction {
    pub fn mock(to: &str, input: Vec<u8>) -> Self {
        Self {
            from: TEST_ADDRESS.to_string(),
            to: Some(to.to_string()),
            input: hex::encode_with_0x(&input),
            call_type: Some("call".to_string()),
        }
    }
}
