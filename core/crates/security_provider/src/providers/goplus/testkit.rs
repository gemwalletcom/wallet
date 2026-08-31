use super::GoPlusProvider;
use gem_client::testkit::MockClient;

impl GoPlusProvider<MockClient> {
    pub fn mock(client: MockClient) -> Self {
        Self::new(client, "", "", None)
    }
}
