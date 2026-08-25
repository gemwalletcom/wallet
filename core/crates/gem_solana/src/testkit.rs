use solana_primitives::{CompiledInstruction, LegacyMessage, MessageHeader, Pubkey, VersionedTransaction};

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

    VersionedTransaction::Legacy {
        signatures: vec![],
        message: LegacyMessage {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: programs.len() as u8,
            },
            account_keys,
            recent_blockhash: [0; 32],
            instructions,
        },
    }
}
