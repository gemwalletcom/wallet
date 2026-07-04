use crate::jsonrpc::TransactionObject;
use crate::testkit::TEST_ADDRESS;

impl TransactionObject {
    pub fn mock(to: &str, value: Option<&str>) -> Self {
        Self {
            from: Some(TEST_ADDRESS.to_string()),
            value: value.map(str::to_string),
            ..Self::new_call(to, vec![])
        }
    }
}
