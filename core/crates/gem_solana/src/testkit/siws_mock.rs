use primitives::testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS;

use crate::siws::SiwsMessage;

impl SiwsMessage {
    pub(crate) fn mock_complete() -> Self {
        Self::parse(include_str!("../../testdata/siws_complete.txt")).unwrap().unwrap()
    }
}

pub(crate) fn mock_siws_message(body: &str) -> String {
    format!("example.com wants you to sign in with your Solana account:\n{TEST_PRIVATE_KEY_SOLANA_ADDRESS}{body}")
}
