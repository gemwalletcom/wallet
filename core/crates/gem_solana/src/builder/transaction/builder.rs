use std::{collections::HashSet, iter::once};

use crate::{
    AccountMeta, AddressLookupTableAccount, Instruction, Message, Pubkey, Result, SignatureBytes, SolanaError, VersionedMessageV0, VersionedTransaction,
    instructions::system::is_advance_nonce_account, types::MAX_ACCOUNT_KEYS,
};

use super::{
    accounts::{AccountBuckets, collect_accounts, compile_instructions, index_accounts},
    lookup::{LoadedAccounts, lookup_locations},
};

#[derive(Debug)]
pub struct TransactionBuilder {
    fee_payer: Pubkey,
    instructions: Vec<Instruction>,
    recent_blockhash: [u8; 32],
}

impl TransactionBuilder {
    pub fn new(fee_payer: Pubkey, recent_blockhash: [u8; 32]) -> Self {
        Self {
            fee_payer,
            instructions: Vec::new(),
            recent_blockhash,
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.instructions.push(instruction);
        self
    }

    pub fn add_instructions(&mut self, instructions: impl IntoIterator<Item = Instruction>) -> &mut Self {
        self.instructions.extend(instructions);
        self
    }

    pub fn build(self) -> Result<VersionedTransaction> {
        let mut account_buckets = AccountBuckets::from_accounts(collect_accounts(self.fee_payer, program_first_accounts(&self.instructions)));
        account_buckets.sort();
        compile_legacy(self.fee_payer, self.recent_blockhash, &account_buckets, &self.instructions)
    }

    pub fn build_v0(self, address_lookup_tables: &[AddressLookupTableAccount]) -> Result<VersionedTransaction> {
        let lookup_locations = lookup_locations(address_lookup_tables);
        let program_ids = self.instructions.iter().map(|instruction| instruction.program_id).collect::<HashSet<_>>();
        let nonce_account = durable_nonce_account(&self.instructions);
        let accounts = collect_accounts(self.fee_payer, program_first_accounts(&self.instructions));
        let mut static_accounts = AccountBuckets::default();
        let mut loaded_accounts = LoadedAccounts::new(address_lookup_tables.len());

        for account in accounts {
            match lookup_locations.get(&account.pubkey).copied() {
                Some(location) if !account.is_signer && !program_ids.contains(&account.pubkey) && Some(account.pubkey) != nonce_account => loaded_accounts.push(account, location),
                _ => static_accounts.push(account),
            }
        }

        let account_keys = static_accounts.account_keys(self.fee_payer);
        if account_keys.len().saturating_add(loaded_accounts.len()) > MAX_ACCOUNT_KEYS {
            return Err(SolanaError::InvalidMessage);
        }

        let header = static_accounts.header()?;
        let mut account_indexes = index_accounts(&account_keys)?;
        for (pubkey, _) in loaded_accounts.iter() {
            let index = account_indexes.len() as u8;
            account_indexes.insert(*pubkey, index);
        }

        let instructions = compile_instructions(&self.instructions, &account_indexes)?;
        let address_table_lookups = loaded_accounts.message_lookups(address_lookup_tables);
        let signatures = vec![SignatureBytes::default(); header.num_required_signatures as usize];

        Ok(VersionedTransaction::V0 {
            signatures,
            message: VersionedMessageV0 {
                message: Message {
                    header,
                    account_keys,
                    recent_blockhash: self.recent_blockhash,
                    instructions,
                },
                address_table_lookups,
            },
        })
    }

    pub fn build_v0_transaction(
        fee_payer: Pubkey,
        recent_blockhash: [u8; 32],
        instructions: &[Instruction],
        address_lookup_tables: &[AddressLookupTableAccount],
    ) -> Result<VersionedTransaction> {
        let mut builder = Self::new(fee_payer, recent_blockhash);
        builder.add_instructions(instructions.iter().cloned());
        builder.build_v0(address_lookup_tables)
    }
}

fn durable_nonce_account(instructions: &[Instruction]) -> Option<Pubkey> {
    let instruction = instructions.first().filter(|instruction| is_advance_nonce_account(instruction))?;
    instruction.accounts.first().map(|account| account.pubkey)
}

fn program_first_accounts(instructions: &[Instruction]) -> impl Iterator<Item = AccountMeta> + '_ {
    instructions
        .iter()
        .flat_map(|instruction| once(AccountMeta::new_readonly(instruction.program_id)).chain(instruction.accounts.iter().cloned()))
}

pub(crate) fn compile_legacy(fee_payer: Pubkey, recent_blockhash: [u8; 32], account_buckets: &AccountBuckets, instructions: &[Instruction]) -> Result<VersionedTransaction> {
    let account_keys = account_buckets.account_keys(fee_payer);
    let account_indexes = index_accounts(&account_keys)?;
    let header = account_buckets.header()?;
    let signatures = vec![SignatureBytes::default(); header.num_required_signatures as usize];
    let instructions = compile_instructions(instructions, &account_indexes)?;

    Ok(VersionedTransaction::Legacy {
        signatures,
        message: Message {
            header,
            account_keys,
            recent_blockhash,
            instructions,
        },
    })
}

#[cfg(test)]
mod tests {
    use gem_encoding::decode_base64;
    use hex_lit::hex;

    use super::TransactionBuilder;
    use crate::{
        AccountMeta, AddressLookupTableAccount, CompiledInstruction, Instruction, MessageAddressTableLookup, Pubkey, SignatureBytes, SolanaError, VersionedTransaction,
        builder::InstructionBuilder,
        instructions::{
            program_ids::{system_program, token_program},
            system::{ADVANCE_NONCE_ACCOUNT_DISCRIMINANT, transfer},
            token::transfer_checked,
        },
        testkit::{TEST_BLOCKHASH, mock_v0_transaction},
    };

    #[test]
    fn test_legacy_transaction_roundtrip() {
        let fee_payer = Pubkey::mock(1);
        let instruction = InstructionBuilder::new(Pubkey::mock(2))
            .account(fee_payer, true, true)
            .account(Pubkey::mock(3), false, true)
            .account(system_program(), false, false)
            .data(vec![4, 5, 6])
            .build();

        let mut builder = TransactionBuilder::new(fee_payer, TEST_BLOCKHASH);
        builder.add_instruction(instruction);
        let transaction = builder.build().unwrap();
        let wire_bytes = transaction.serialize().unwrap();
        let decoded = VersionedTransaction::deserialize_with_version(&wire_bytes).unwrap();
        assert_eq!(decoded, transaction);
    }

    #[test]
    fn test_legacy_transaction_with_native_and_token_transfers() {
        let payer = Pubkey::mock(1);
        let recipient = Pubkey::mock(2);
        let source = Pubkey::mock(3);
        let owner = Pubkey::mock(4);
        let mint = Pubkey::mock(5);
        let mut builder = TransactionBuilder::new(payer, TEST_BLOCKHASH);

        let lamports = 1_000_000_000;
        builder.add_instruction(transfer(&payer, &recipient, lamports));

        let amount = 1_000_000;
        let decimals = 6;

        let token_transfer = transfer_checked(&source, &mint, &recipient, &owner, amount, decimals);
        builder.add_instruction(token_transfer);

        let transaction = builder.build().unwrap();

        assert_eq!(transaction.signatures().len(), 2);
        assert_eq!(transaction.account_keys(), vec![payer, owner, recipient, source, system_program(), mint, token_program()]);
        assert_eq!(transaction.instructions().len(), 2);
    }

    #[test]
    fn test_versioned_transaction_builder_without_lookup_tables() {
        let fee_payer = Pubkey::mock(1);
        let recipient = Pubkey::mock(2);

        let transfer_instruction = transfer(&fee_payer, &recipient, 123);

        let mut builder = TransactionBuilder::new(fee_payer, TEST_BLOCKHASH);
        builder.add_instruction(transfer_instruction);

        let transaction = builder.build_v0(&[]).unwrap();
        let wire_bytes = transaction.serialize().unwrap();
        let parsed = VersionedTransaction::deserialize_with_version(&wire_bytes).unwrap();

        let mut data = 2u32.to_le_bytes().to_vec();
        data.extend_from_slice(&123u64.to_le_bytes());
        assert_eq!(
            parsed,
            mock_v0_transaction(vec![fee_payer, recipient, system_program()], vec![CompiledInstruction::mock(2, vec![0, 1], data)], vec![],)
        );
    }

    #[test]
    fn test_versioned_transaction_builder_keeps_the_nonce_account_static() {
        let fee_payer = Pubkey::mock(1);
        let nonce_account = Pubkey::mock(2);
        let looked_up_account = Pubkey::mock(3);
        let advance_nonce = InstructionBuilder::new(system_program())
            .account(nonce_account, false, true)
            .account(looked_up_account, false, false)
            .data(ADVANCE_NONCE_ACCOUNT_DISCRIMINANT.to_vec())
            .build();
        let lookup_table = AddressLookupTableAccount::new(Pubkey::mock(4), vec![nonce_account, looked_up_account]);

        let mut builder = TransactionBuilder::new(fee_payer, TEST_BLOCKHASH);
        builder.add_instruction(advance_nonce);
        let transaction = builder.build_v0(&[lookup_table]).unwrap();

        assert_eq!(
            transaction,
            mock_v0_transaction(
                vec![fee_payer, nonce_account, system_program()],
                vec![CompiledInstruction::mock(2, vec![1, 3], ADVANCE_NONCE_ACCOUNT_DISCRIMINANT.to_vec())],
                vec![MessageAddressTableLookup::new(Pubkey::mock(4), vec![], vec![1])],
            )
        );
    }

    #[test]
    fn test_versioned_transaction_builder_with_lookup_table() {
        let fee_payer = Pubkey::mock(1);
        let looked_up_account = Pubkey::mock(2);
        let program_id = Pubkey::mock(3);

        let instruction = InstructionBuilder::new(program_id)
            .account(fee_payer, true, true)
            .account(looked_up_account, false, true)
            .data(vec![1, 2, 3])
            .build();

        let lookup_table = AddressLookupTableAccount::new(Pubkey::mock(4), vec![looked_up_account, Pubkey::mock(5)]);

        let mut builder = TransactionBuilder::new(fee_payer, TEST_BLOCKHASH);
        builder.add_instruction(instruction);

        let transaction = builder.build_v0(&[lookup_table]).unwrap();
        let wire_bytes = transaction.serialize().unwrap();
        let parsed = VersionedTransaction::deserialize_with_version(&wire_bytes).unwrap();

        assert_eq!(
            parsed,
            mock_v0_transaction(
                vec![fee_payer, program_id],
                vec![CompiledInstruction::mock(1, vec![0, 2], vec![1, 2, 3])],
                vec![MessageAddressTableLookup::new(Pubkey::mock(4), vec![0], vec![])],
            )
        );
    }

    #[test]
    fn test_v0_builder_real_world_regression_case() {
        const REAL_TRANSACTION_BASE64: &str = "AlF6Dlk4UjQD0xek1R2X8/hcORMjfzZ7/Vmql3hZcmM3+wwWrtvNkbqDFGZqJyFQxlNopEYLGJ3Oo/9gTDqylwOaaKU6sUi0z0x/4AIr2bEbk4F0Bb3eQnlZB2Pd4fwON80kvuBSbQPthCRffekiFXCnIXQUNFcuW3YDiZP0o0oBgAIACA2mI04pxqQuMUitv1NuRlK9ZWJWaV1k+p/LfT3tvKJ+fbIxWsd0GlHg175uFfLQ+Y+1DxMT48DDYU+4V77WYfZ1G4LkfQewG7EXCfCqmCEkGyByWhJU1GOFbK7yr0N338lnQQQP5AeqsFBGoH5xsx9hmNdlxN72v4J91uC6Ksvw/j23WlYbqpa0+YWZyJHXFuu3ghb5vWc1zPY3lpthsJywjJclj04kifG7PRApFI4NgwtaE5na/xCEBI572Nvp+FkAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpBHnVW/IxwG7udMVuzmgVB/2xst6j9I5RArHNola8E49RPixdukLDvYMw2r2DTumX5VA1ifoAVfXgkOTnLswDErQ/+if11/ZKdMCbHylYed5LCas238ndUUsyGqezjOXot/ord4dFsTTM4tnKRq1DlX7l6IZI9NWIfD9sANaw+DcFSlNamSkhBk0k6HFg2jh8fDW13bySu4HkH6hAQQVEjeSccqqE9cRgyD3i0H5PnVvX+q6L+uN2Xdbz16thksnaBgUGABAREwYHAQEGAgECDAIAAACApL8HAAAAAAcBAgERCBoHCQECAwQQFBMICAoIFRYNCQ4PAwQUEwcHFyXBIJszQdacgQMBAAAAWQFkAAGApL8HAAAAAGzHtAAAAAAAyAAACwwAAREYGRMQEhoHBQYonVNwIb8yqyWioGpTHjinCEcrRIIzlc3YWKd5g/z9UQsz2pcScMC7SQwAQjB4YzBiZDEyNDczNjVlM2Q2MTMyM2IxYTYyM2YwMzI0MDUzMzU0Yjk2MTJhODkzNTg3YjdlYTMyNjNlN2JhMTNiNAJ5QE4t+Dvx0UlyGT++v3V9s/1gQI0crEMfwbwNXZBmFgPb3xcF3gjcFuH5CleX0p1W2E0BwNC64/nFjEaXTuuVyg5P1Sf64f/vAgMMAwcDAQIA";
        const STATIC_KEYS: [&str; 13] = [
            "CBXuKTC3JAHjCvUeCXF2mXJazBqATDExQRxZi1iqQcDa",
            "CzbDjxK4wqSpBuKfocC9vUgpzfhVEGPh8EihXbQkophA",
            "2rPmeokZcYM8F3roghsoPYNqSYi32QyrNA9Lm7gN8TDa",
            "7x4VcEX8aLd3kFsNWULTp1qFgVtDwyWSxpTGQkoMM6XX",
            "59v2cSbCsnyaWymLnsq6TWzE6cEN5KJYNTBNrcP4smRH",
            "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
            "11111111111111111111111111111111",
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
            "6U91aKa8pmMxkJwBCfPTmUEfZi6dHe7DcFq2ALvB2tbB",
            "D8cy77BBepLMngZx6ZukaTff5hCt1HrWyKk3Hnd9oitf",
            "DPArtTLbEqa6EuXHfL5UFLBZhFjiEXWRudhvXDrjwXUr",
            "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        ];
        const LOADED_WRITABLE: [&str; 6] = [
            "FLckHLGMJy5gEoXWwcE68Nprde1D4araK4TGLw4pQq2n",
            "5pVN5XZB8cYBjNLFrsBCPWkCQBan5K5Mq2dWGzwPgGJV",
            "9t4P5wMwfFkyn92Z7hf463qYKEZf8ERVZsGBEPNp8uJx",
            "H2DG3qk1cRqBUmRNjJ2fsGrGs47NQk5VRBLt1AevW8m2",
            "6GpvpHXBJA7pW8gP9KEXBJ2spNyydmbY5Q4nbdoo5TeT",
            "4nvJ5zWdVspxJiNZzB127U6amPH98SFFkBx2JZrAduia",
        ];
        const LOADED_READONLY: [&str; 8] = [
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "So11111111111111111111111111111111111111112",
            "TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH",
            "8ekCy2jHHUbW2yeNGFWYJT9Hm9FW7SvZcZK66dSZCDiF",
            "Sysvar1nstructions1111111111111111111111111",
            "Dodg2HifwU8rmaVVyMyUZDGTRbqAJTyVYxXPwcbNpBKc",
            "7uTT8Xi5RWXzy7h9XL244GRgEycDYDhLjr3ZyNdXi8pZ",
            "99vQwtBwYtrqqD9YSXbdum3KBdxPAVxYTaQ3cfnJSrN2",
        ];

        let wire = decode_base64(REAL_TRANSACTION_BASE64).unwrap();
        let mut expected_transaction = VersionedTransaction::deserialize_with_version(&wire).unwrap();
        for signature in expected_transaction.signatures_mut() {
            *signature = SignatureBytes::default();
        }

        let fee_payer = expected_transaction.account_keys()[0];
        let recent_blockhash = *expected_transaction.recent_blockhash();

        let combined_accounts: Vec<AccountMeta> = STATIC_KEYS
            .iter()
            .chain(LOADED_WRITABLE.iter())
            .chain(LOADED_READONLY.iter())
            .copied()
            .map(|value| Pubkey::from_base58(value).unwrap())
            .enumerate()
            .map(|(index, key)| {
                let is_signer = index < 2;
                let is_writable = index < 2 || (2..5).contains(&index) || (13..19).contains(&index);
                AccountMeta::new(key, is_signer, is_writable)
            })
            .collect();

        let lookup_tables = vec![
            AddressLookupTableAccount::mock(
                "9AKCoNoAGYLW71TwTHY9e7KrZUWWL3c7VtHKb66NT3EV",
                &[
                    (219, "FLckHLGMJy5gEoXWwcE68Nprde1D4araK4TGLw4pQq2n"),
                    (223, "5pVN5XZB8cYBjNLFrsBCPWkCQBan5K5Mq2dWGzwPgGJV"),
                    (23, "9t4P5wMwfFkyn92Z7hf463qYKEZf8ERVZsGBEPNp8uJx"),
                    (222, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
                    (8, "So11111111111111111111111111111111111111112"),
                    (220, "TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH"),
                    (22, "8ekCy2jHHUbW2yeNGFWYJT9Hm9FW7SvZcZK66dSZCDiF"),
                    (225, "Sysvar1nstructions1111111111111111111111111"),
                ],
            ),
            AddressLookupTableAccount::mock(
                "Hm9fUgcn7qwDaiNTFiGh6pNtVATgnaRcmK6Bbx6EMZfP",
                &[
                    (12, "H2DG3qk1cRqBUmRNjJ2fsGrGs47NQk5VRBLt1AevW8m2"),
                    (3, "6GpvpHXBJA7pW8gP9KEXBJ2spNyydmbY5Q4nbdoo5TeT"),
                    (7, "4nvJ5zWdVspxJiNZzB127U6amPH98SFFkBx2JZrAduia"),
                    (1, "Dodg2HifwU8rmaVVyMyUZDGTRbqAJTyVYxXPwcbNpBKc"),
                    (2, "7uTT8Xi5RWXzy7h9XL244GRgEycDYDhLjr3ZyNdXi8pZ"),
                    (0, "99vQwtBwYtrqqD9YSXbdum3KBdxPAVxYTaQ3cfnJSrN2"),
                ],
            ),
        ];

        let instructions = vec![
            Instruction::mock(5, &hex!("001011130607"), "2", &combined_accounts),
            Instruction::mock(6, &[1, 2], "3Bxs4NNfTBw5NH5H", &combined_accounts),
            Instruction::mock(7, &[2], "J", &combined_accounts),
            Instruction::mock(
                8,
                &hex!("07090102030410141308080a0815160d090e0f03041413070717"),
                "7UR2vxkjV6WhbmWvkCZQvQJKVhT964yPqVRoTBAPv678iyHS8LF",
                &combined_accounts,
            ),
            Instruction::mock(
                11,
                &hex!("00011118191310121a070506"),
                "8pPpkivb1mTLA5APTUWQU2CsG1oYxcnh5C8fQsYux2BVVxSXuFvWtLx",
                &combined_accounts,
            ),
            Instruction::mock(
                12,
                &[],
                "KszMTKrqxdHWZULtjrD9cmodXEnC1UboEfkgMRLSGuuPLDYWo8BrqcbfRddG4w18gsf1sZR69vK1mKhXyvNCTZxwsq",
                &combined_accounts,
            ),
        ];

        let mut builder = TransactionBuilder::new(fee_payer, recent_blockhash);
        builder.add_instructions(instructions);
        let rebuilt_transaction = builder.build_v0(&lookup_tables).unwrap();

        assert_eq!(rebuilt_transaction.serialize().unwrap(), expected_transaction.serialize().unwrap());
    }

    #[test]
    fn test_build_rejects_more_than_256_distinct_accounts() {
        let fee_payer = Pubkey::mock(1);
        let program_id = Pubkey::mock(2);

        let accounts: Vec<AccountMeta> = (3..=257).map(|index| AccountMeta::new_writable(Pubkey::mock(index))).collect();

        let instruction = Instruction {
            program_id,
            accounts,
            data: vec![],
        };

        let mut builder = TransactionBuilder::new(fee_payer, TEST_BLOCKHASH);
        builder.add_instruction(instruction);

        let result = builder.build();
        assert_eq!(result.unwrap_err(), SolanaError::InvalidMessage);
    }

    #[test]
    fn test_build_rejects_256_required_signers() {
        let fee_payer = Pubkey::mock(1);
        let signer_pubkeys: Vec<Pubkey> = (2..=256).map(Pubkey::mock).collect();
        let program_id = signer_pubkeys[0];

        let accounts: Vec<AccountMeta> = signer_pubkeys.iter().map(|pubkey| AccountMeta::new_signer_writable(*pubkey)).collect();

        let instruction = Instruction {
            program_id,
            accounts,
            data: vec![],
        };

        let mut builder = TransactionBuilder::new(fee_payer, TEST_BLOCKHASH);
        builder.add_instruction(instruction);

        let result = builder.build();
        assert_eq!(result.unwrap_err(), SolanaError::InvalidMessage);
    }
}
