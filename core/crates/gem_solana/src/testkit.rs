use primitives::testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS;
use solana_primitives::{CompiledInstruction, LegacyMessage, MessageHeader, Pubkey, VersionedTransaction};

use crate::siws::SiwsMessage;

pub(crate) fn mock_transaction(programs: &[(&str, Vec<u8>)]) -> VersionedTransaction {
    let mut account_keys = vec![Pubkey::new([1; 32])];
    account_keys.extend(programs.iter().map(|(program, _)| Pubkey::from_base58(program).unwrap()));
    let instructions = programs
        .iter()
        .enumerate()
        .map(|(index, (_, data))| CompiledInstruction {
            program_id_index: (index + 1) as u8,
            accounts: vec![],
            data: data.clone(),
        })
        .collect();

    mock_transaction_with_accounts(account_keys, instructions)
}

pub(crate) fn mock_transaction_with_accounts(account_keys: Vec<Pubkey>, instructions: Vec<CompiledInstruction>) -> VersionedTransaction {
    VersionedTransaction::Legacy {
        signatures: vec![],
        message: LegacyMessage {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: (account_keys.len() - 1) as u8,
            },
            account_keys,
            recent_blockhash: [0; 32],
            instructions,
        },
    }
}

impl SiwsMessage {
    pub(crate) fn mock_complete() -> Self {
        Self::parse(include_str!("../testdata/siws_complete.txt")).unwrap().unwrap()
    }
}

pub(crate) fn mock_siws_message(body: &str) -> String {
    format!("example.com wants you to sign in with your Solana account:\n{TEST_PRIVATE_KEY_SOLANA_ADDRESS}{body}")
}
