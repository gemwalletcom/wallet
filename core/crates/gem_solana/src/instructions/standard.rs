use crate::{
    CompiledInstruction, Instruction, Pubkey,
    instructions::{
        associated_token::is_create_account_data,
        compute_budget,
        program_ids::{SOLANA_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID, SOLANA_COMPUTE_BUDGET_PROGRAM_ID, SOLANA_MEMO_PROGRAM_ID, SOLANA_SYSTEM_PROGRAM_ID, SOLANA_TOKEN_2022_PROGRAM_ID, SOLANA_TOKEN_PROGRAM_ID},
        system, token,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardInstruction<'a> {
    ComputeBudget {
        unit_price: Option<u64>,
    },
    Memo,
    Transfer {
        source: Option<&'a Pubkey>,
        destination: Option<&'a Pubkey>,
        lamports: u64,
    },
    CreateTokenAccount {
        account: Option<&'a Pubkey>,
        owner: Option<&'a Pubkey>,
        token_program: Option<&'a Pubkey>,
    },
    SyncNative,
    CloseAccount {
        account: Option<&'a Pubkey>,
        destination: Option<&'a Pubkey>,
    },
    TransferChecked {
        mint: Option<&'a Pubkey>,
        destination: Option<&'a Pubkey>,
        authority: Option<&'a Pubkey>,
        amount: u64,
    },
    Program(&'a Pubkey),
    Unsupported,
}

impl<'a> StandardInstruction<'a> {
    pub fn new(instruction: &'a Instruction) -> Self {
        Self::decode(&instruction.program_id, |position| instruction.account(position), &instruction.data)
    }

    pub fn compiled(instruction: &'a CompiledInstruction, keys: &'a [Pubkey]) -> Option<Self> {
        let program = keys.get(usize::from(instruction.program_id_index))?;
        Some(Self::decode(program, |position| keys.get(usize::from(*instruction.accounts.get(position)?)), &instruction.data))
    }

    fn decode(program: &'a Pubkey, account: impl Fn(usize) -> Option<&'a Pubkey>, data: &[u8]) -> Self {
        match program.to_base58().as_str() {
            SOLANA_COMPUTE_BUDGET_PROGRAM_ID => Self::ComputeBudget {
                unit_price: compute_budget::parse_compute_unit_price_data(data),
            },
            SOLANA_MEMO_PROGRAM_ID => Self::Memo,
            SOLANA_SYSTEM_PROGRAM_ID => system::transfer_lamports(data).map_or(Self::Unsupported, |lamports| Self::Transfer {
                source: account(0),
                destination: account(1),
                lamports,
            }),
            SOLANA_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID => match is_create_account_data(data) {
                true => Self::CreateTokenAccount {
                    account: account(1),
                    owner: account(2),
                    token_program: account(5),
                },
                false => Self::Unsupported,
            },
            SOLANA_TOKEN_PROGRAM_ID | SOLANA_TOKEN_2022_PROGRAM_ID => match data {
                [token::SYNC_NATIVE_DISCRIMINANT] => Self::SyncNative,
                [token::CLOSE_ACCOUNT_DISCRIMINANT] => Self::CloseAccount {
                    account: account(0),
                    destination: account(1),
                },
                _ => token::transfer_checked_amount(data).map_or(Self::Unsupported, |amount| Self::TransferChecked {
                    mint: account(1),
                    destination: account(2),
                    authority: account(3),
                    amount,
                }),
            },
            _ => Self::Program(program),
        }
    }

    pub fn transfers_to(&self, account: &Pubkey) -> bool {
        match self {
            Self::Transfer { destination, .. } => *destination == Some(account),
            Self::ComputeBudget { .. } | Self::Memo | Self::CreateTokenAccount { .. } | Self::SyncNative | Self::CloseAccount { .. } | Self::TransferChecked { .. } | Self::Program(_) | Self::Unsupported => false,
        }
    }

    pub fn closes_to(&self, account: &Pubkey, destination: &Pubkey) -> bool {
        match self {
            Self::CloseAccount { account: closed, destination: paid } => *closed == Some(account) && *paid == Some(destination),
            Self::ComputeBudget { .. } | Self::Memo | Self::Transfer { .. } | Self::CreateTokenAccount { .. } | Self::SyncNative | Self::TransferChecked { .. } | Self::Program(_) | Self::Unsupported => false,
        }
    }

    pub(crate) fn created_account_owner(&self, account: &Pubkey) -> Option<&'a Pubkey> {
        match self {
            Self::CreateTokenAccount { account: created, owner, .. } if *created == Some(account) => *owner,
            Self::ComputeBudget { .. } | Self::Memo | Self::Transfer { .. } | Self::CreateTokenAccount { .. } | Self::SyncNative | Self::CloseAccount { .. } | Self::TransferChecked { .. } | Self::Program(_) | Self::Unsupported => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AccountMeta,
        instructions::{
            associated_token::create_associated_token_account_idempotent,
            program_ids::{system_program, token_2022_program, token_program},
        },
    };

    #[test]
    fn test_decode() {
        let wallet = Pubkey::new([1; 32]);
        let account = Pubkey::new([2; 32]);
        let mint = Pubkey::new([3; 32]);
        let program = Pubkey::new([4; 32]);
        let created = create_associated_token_account_idempotent(&wallet, &wallet, &mint, &token_2022_program()).unwrap();
        let transfer = system::transfer(&wallet, &account, 5_000);
        let token_transfer = token::transfer_checked_with_program_id(&account, &mint, &wallet, &program, 7, 6, &token_2022_program());
        let close = token::close_account(&account, &wallet, &wallet);
        let sync = token::sync_native(&account);
        let price = compute_budget::set_compute_unit_price(9);
        let limit = compute_budget::set_compute_unit_limit(200_000);
        let unknown = Instruction {
            program_id: program,
            accounts: vec![AccountMeta::new_writable(account)],
            data: vec![1],
        };

        assert_eq!(
            StandardInstruction::new(&created),
            StandardInstruction::CreateTokenAccount {
                account: created.account(1),
                owner: Some(&wallet),
                token_program: Some(&token_2022_program()),
            }
        );
        assert_eq!(
            StandardInstruction::new(&transfer),
            StandardInstruction::Transfer {
                source: Some(&wallet),
                destination: Some(&account),
                lamports: 5_000,
            }
        );
        assert_eq!(
            StandardInstruction::new(&token_transfer),
            StandardInstruction::TransferChecked {
                mint: Some(&mint),
                destination: Some(&wallet),
                authority: Some(&program),
                amount: 7,
            }
        );
        assert_eq!(
            StandardInstruction::new(&close),
            StandardInstruction::CloseAccount {
                account: Some(&account),
                destination: Some(&wallet),
            }
        );
        assert_eq!(StandardInstruction::new(&sync), StandardInstruction::SyncNative);
        assert_eq!(StandardInstruction::new(&price), StandardInstruction::ComputeBudget { unit_price: Some(9) });
        assert_eq!(StandardInstruction::new(&limit), StandardInstruction::ComputeBudget { unit_price: None });
        assert_eq!(StandardInstruction::new(&unknown), StandardInstruction::Program(&program));

        for (instruction, reason) in [
            (
                Instruction {
                    data: [transfer.data.as_slice(), &[0]].concat(),
                    ..transfer.clone()
                },
                "a system transfer with trailing data",
            ),
            (
                Instruction {
                    data: token_transfer.data[..9].to_vec(),
                    ..token_transfer.clone()
                },
                "a token transfer without its decimals",
            ),
            (
                Instruction {
                    data: [token_transfer.data.as_slice(), &[0]].concat(),
                    ..token_transfer.clone()
                },
                "a token transfer with trailing data",
            ),
            (token::transfer(&account, &wallet, &wallet, 1), "a token transfer that does not name its mint"),
            (Instruction { data: vec![2], ..created.clone() }, "an associated token instruction that does not create an account"),
            (
                Instruction {
                    program_id: system_program(),
                    ..unknown.clone()
                },
                "a system instruction other than a transfer",
            ),
            (
                Instruction {
                    program_id: token_program(),
                    data: vec![6],
                    ..unknown
                },
                "a token instruction the swap and transfer checks do not read",
            ),
        ] {
            assert_eq!(StandardInstruction::new(&instruction), StandardInstruction::Unsupported, "{reason}");
        }
    }

    #[test]
    fn test_compiled() {
        let keys = [Pubkey::new([1; 32]), Pubkey::new([2; 32]), system_program()];
        let transfer = CompiledInstruction::mock(2, vec![0, 1], system::transfer(&keys[0], &keys[1], 5).data);
        let from_lookup_table = CompiledInstruction::mock(2, vec![0, 3], transfer.data.clone());

        assert_eq!(
            StandardInstruction::compiled(&transfer, &keys),
            Some(StandardInstruction::Transfer {
                source: Some(&keys[0]),
                destination: Some(&keys[1]),
                lamports: 5,
            })
        );
        assert_eq!(
            StandardInstruction::compiled(&from_lookup_table, &keys),
            Some(StandardInstruction::Transfer {
                source: Some(&keys[0]),
                destination: None,
                lamports: 5,
            }),
            "an account loaded from a lookup table cannot be read"
        );
        assert_eq!(StandardInstruction::compiled(&CompiledInstruction::mock(3, vec![], vec![]), &keys), None, "a program loaded from a lookup table");
    }
}
