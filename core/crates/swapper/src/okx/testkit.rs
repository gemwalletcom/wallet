use super::{OkxClientConfig, OkxProvider, model::TransactionData};
use crate::alien::mock::ProviderMock;
use gem_client::testkit::MockClient;
use std::sync::Arc;

impl TransactionData {
    pub fn mock(to: &str, value: &str, data: &str, gas: &str) -> Self {
        Self {
            data: data.to_string(),
            to: to.to_string(),
            value: value.to_string(),
            gas: gas.to_string(),
            signature_data: None,
        }
    }
}

impl OkxProvider<MockClient> {
    pub fn mock(rpc_result: &str) -> Self {
        let config = OkxClientConfig {
            api_key: String::new(),
            secret_key: String::new(),
            passphrase: String::new(),
            project: String::new(),
        };
        Self::new_with_client(MockClient::new(), config, Arc::new(ProviderMock::new(rpc_result.to_string())))
    }
}
