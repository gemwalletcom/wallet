use crate::client::JupiterClient;
use gem_client::testkit::MockClient;

impl JupiterClient<MockClient> {
    pub fn mock(response: &'static str) -> Self {
        Self::new_with_client(MockClient::new().with_get(move |_| Ok(response.as_bytes().to_vec())))
    }
}
