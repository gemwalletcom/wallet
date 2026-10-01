use crate::{CompiledInstruction, Message, MessageHeader, Pubkey, TransactionConfig, VersionedMessageV1, testkit::TEST_BLOCKHASH};

impl MessageHeader {
    pub(crate) fn mock(num_required_signatures: u8, num_readonly_unsigned_accounts: u8) -> Self {
        Self {
            num_required_signatures,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts,
        }
    }
}

impl TransactionConfig {
    pub(crate) fn mock() -> Self {
        Self {
            priority_fee: Some(10_000),
            compute_unit_limit: Some(200_000),
            loaded_accounts_data_size_limit: Some(8192),
            heap_size: Some(32 * 1024),
        }
    }
}

impl VersionedMessageV1 {
    pub(crate) fn mock(num_required_signatures: u8, account_keys: Vec<Pubkey>, instructions: Vec<CompiledInstruction>, config: TransactionConfig) -> Self {
        Self {
            message: Message {
                header: MessageHeader::mock(num_required_signatures, account_keys.len() as u8 - num_required_signatures),
                account_keys,
                recent_blockhash: TEST_BLOCKHASH,
                instructions,
            },
            config,
        }
    }
}
