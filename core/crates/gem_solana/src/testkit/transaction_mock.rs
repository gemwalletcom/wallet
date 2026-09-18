use crate::{
    CompiledInstruction, Message, MessageAddressTableLookup, MessageHeader, Pubkey, SignatureBytes, TransactionConfig, VersionedMessageV0, VersionedMessageV1, VersionedTransaction,
    testkit::{TEST_BLOCKHASH, test_wallet_pubkey},
};

pub(crate) fn mock_transaction(programs: &[(&str, Vec<u8>)]) -> VersionedTransaction {
    let mut account_keys = vec![Pubkey::new([1; 32])];
    account_keys.extend(programs.iter().map(|(program, _)| Pubkey::from_base58(program).unwrap()));
    let instructions = programs.iter().enumerate().map(|(index, (_, data))| CompiledInstruction::mock((index + 1) as u8, vec![], data.clone())).collect();

    mock_transaction_with_accounts(account_keys, instructions)
}

#[cfg(feature = "signer")]
pub(crate) fn mock_legacy_transaction() -> VersionedTransaction {
    mock_transaction_with_accounts(vec![test_wallet_pubkey(), Pubkey::new([2; 32])], vec![CompiledInstruction::mock(1, vec![0], vec![])])
}

pub(crate) fn mock_transaction_with_accounts(account_keys: Vec<Pubkey>, instructions: Vec<CompiledInstruction>) -> VersionedTransaction {
    VersionedTransaction::Legacy {
        signatures: vec![],
        message: Message {
            header: MessageHeader::mock(1, (account_keys.len() - 1) as u8),
            account_keys,
            recent_blockhash: [0; 32],
            instructions,
        },
    }
}

pub(crate) fn mock_v0_transaction(account_keys: Vec<Pubkey>, instructions: Vec<CompiledInstruction>, address_table_lookups: Vec<MessageAddressTableLookup>) -> VersionedTransaction {
    VersionedTransaction::V0 {
        signatures: vec![SignatureBytes::default()],
        message: VersionedMessageV0 {
            message: Message {
                header: MessageHeader::mock(1, 1),
                account_keys,
                recent_blockhash: TEST_BLOCKHASH,
                instructions,
            },
            address_table_lookups,
        },
    }
}

pub(crate) fn mock_v1_transaction(signature_count: u8, wallet_index: u8) -> VersionedTransaction {
    let mut account_keys = (1..=signature_count).map(|value| Pubkey::new([value; 32])).collect::<Vec<_>>();
    if let Some(account) = account_keys.get_mut(wallet_index as usize) {
        *account = test_wallet_pubkey();
    }
    account_keys.push(Pubkey::new([100; 32]));
    VersionedTransaction::V1 {
        signatures: (0..signature_count).map(|value| if value == wallet_index { SignatureBytes::default() } else { SignatureBytes::new([value + 1; 64]) }).collect(),
        message: VersionedMessageV1::mock(signature_count, account_keys, vec![CompiledInstruction::mock(signature_count, vec![0], vec![0xde, 0xad])], TransactionConfig::mock()),
    }
}
