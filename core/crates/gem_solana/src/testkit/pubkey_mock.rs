use primitives::testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS;

use crate::Pubkey;

impl Pubkey {
    pub(crate) fn mock(value: u32) -> Self {
        let mut bytes = [0; 32];
        bytes[..4].copy_from_slice(&value.to_le_bytes());
        Self::new(bytes)
    }
}

pub(crate) fn test_wallet_pubkey() -> Pubkey {
    Pubkey::from_base58(TEST_PRIVATE_KEY_SOLANA_ADDRESS).unwrap()
}
