use crate::client::JupiterClient;
use gem_client::testkit::MockClient;

pub(crate) const TOKEN_SEARCH_RESULTS: &str = r#"[
    {"id":"verified","isVerified":true,"audit":null},
    {"id":"unverified","isVerified":false,"audit":{}},
    {"id":"suspicious","isVerified":null,"audit":{"isSus":false}}
]"#;

impl JupiterClient<MockClient> {
    pub fn mock(response: &'static str) -> Self {
        Self::new_with_client(MockClient::new().with_get(move |_| Ok(response.as_bytes().to_vec())))
    }
}
