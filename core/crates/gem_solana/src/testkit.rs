use primitives::testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS;

use num_bigint::BigUint;

use crate::{
    AccountMeta, AddressLookupTableAccount, CompiledInstruction, Instruction, Message, MessageHeader, Pubkey, VersionedTransaction,
    models::{
        BlockTransaction, EpochInfo, Info, Meta, Parsed, StakeDelegation, StakeInfo, TokenAccountData, TokenAccountInfo, TokenAccountInfoData, TokenAmount, TokenBalance,
        Transaction, TransactionMessage,
    },
    siws::SiwsMessage,
};

pub(crate) const TEST_BLOCKHASH: [u8; 32] = [1; 32];

impl Pubkey {
    pub(crate) fn mock(value: u32) -> Self {
        let mut bytes = [0; 32];
        bytes[..4].copy_from_slice(&value.to_le_bytes());
        Self::new(bytes)
    }
}

impl AddressLookupTableAccount {
    pub(crate) fn mock(table_key: &str, entries: &[(u8, &str)]) -> Self {
        let max_index = entries.iter().map(|(index, _)| *index).max().unwrap_or(0) as usize;
        let mut addresses: Vec<Pubkey> = (0..=max_index)
            .map(|entry_index| {
                let mut bytes = [0; 32];
                bytes[0] = 0xFE;
                bytes[1..3].copy_from_slice(&(entry_index as u16).to_le_bytes());
                Pubkey::new(bytes)
            })
            .collect();

        for (index, value) in entries {
            addresses[*index as usize] = Pubkey::from_base58(value).unwrap();
        }

        Self::new(Pubkey::from_base58(table_key).unwrap(), addresses)
    }
}

impl Instruction {
    pub(crate) fn mock(program_id_index: usize, account_indexes: &[u8], data_base58: &str, combined_accounts: &[AccountMeta]) -> Self {
        Self {
            program_id: combined_accounts[program_id_index].pubkey,
            accounts: account_indexes.iter().map(|index| combined_accounts[*index as usize].clone()).collect(),
            data: bs58::decode(data_base58).into_vec().unwrap(),
        }
    }
}

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

#[cfg(feature = "signer")]
pub(crate) fn mock_legacy_transaction() -> VersionedTransaction {
    mock_transaction_with_accounts(
        vec![Pubkey::new([1; 32]), Pubkey::new([2; 32])],
        vec![CompiledInstruction {
            program_id_index: 1,
            accounts: vec![0],
            data: vec![],
        }],
    )
}

pub(crate) fn mock_transaction_with_accounts(account_keys: Vec<Pubkey>, instructions: Vec<CompiledInstruction>) -> VersionedTransaction {
    VersionedTransaction::Legacy {
        signatures: vec![],
        message: Message {
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

impl EpochInfo {
    pub fn mock(slot_index: u64) -> Self {
        EpochInfo {
            epoch: 200,
            slot_index,
            slots_in_epoch: 432000,
        }
    }
}

impl TokenBalance {
    pub fn mock(mint: &str, owner: &str, amount: u64) -> Self {
        Self {
            account_index: 0,
            mint: mint.to_string(),
            owner: owner.to_string(),
            ui_token_amount: TokenAmount {
                amount: BigUint::from(amount),
                decimals: 6,
            },
        }
    }
}

impl TokenAccountInfo {
    pub fn mock_stake(activation_epoch: u64, deactivation_epoch: u64) -> Self {
        Self {
            pubkey: "stake1".to_string(),
            account: TokenAccountData {
                data: Parsed {
                    parsed: Info {
                        info: TokenAccountInfoData {
                            mint: None,
                            token_amount: None,
                            stake: Some(StakeInfo {
                                delegation: StakeDelegation {
                                    activation_epoch,
                                    deactivation_epoch,
                                    stake: "1000000".to_string(),
                                    voter: "validator1".to_string(),
                                },
                            }),
                        },
                    },
                },
                owner: "owner1".to_string(),
                lamports: 1000000,
            },
        }
    }
}

impl BlockTransaction {
    pub fn mock(account_keys: &[&str], pre_balances: Vec<u64>, post_balances: Vec<u64>) -> Self {
        Self {
            meta: Meta {
                err: None,
                fee: 5000,
                pre_balances,
                post_balances,
                pre_token_balances: vec![],
                post_token_balances: vec![],
                loaded_addresses: None,
            },
            transaction: Transaction {
                message: TransactionMessage {
                    account_keys: account_keys.iter().map(|key| key.to_string()).collect(),
                    instructions: vec![],
                },
                signatures: vec![],
            },
        }
    }
}
