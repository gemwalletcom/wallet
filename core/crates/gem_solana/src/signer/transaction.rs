use crate::decode_transaction;
use gem_encoding::encode_base64;
use num_traits::ToPrimitive;
use primitives::{SignerError, SignerInput, TransactionFee};
use solana_primitives::{
    CompiledInstruction, Instruction, LegacyMessage, MessageHeader, Pubkey, SignatureBytes, VersionedTransaction,
    instructions::{
        compute_budget::{set_compute_unit_limit, set_compute_unit_price},
        program_ids::system_program,
    },
    sign_message as sign_solana_message,
};
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct AccountFlags {
    is_signer: bool,
    is_writable: bool,
}

pub(crate) fn compute_budget_instructions(fee: &TransactionFee) -> Result<Vec<Instruction>, SignerError> {
    let (unit_price, compute_unit_limit) = compute_budget_values(fee)?;
    let mut instructions = Vec::with_capacity(2);
    if unit_price > 0 {
        instructions.push(set_compute_unit_price(unit_price));
    }
    if compute_unit_limit > 0 {
        instructions.push(set_compute_unit_limit(compute_unit_limit));
    }
    Ok(instructions)
}

pub(crate) fn sign_single_signer_instructions(input: &SignerInput, private_key: &[u8], fee_payer: Pubkey, instructions: Vec<Instruction>) -> Result<String, SignerError> {
    let transaction = build_legacy_transaction(fee_payer, block_hash(input)?, instructions)?;
    if transaction.num_required_signatures() != 1 {
        return Err(SignerError::invalid_input("Solana transaction requires more than one signer"));
    }
    sign_and_encode_transaction(transaction, private_key)
}

pub(super) fn sign_transaction(transaction: &VersionedTransaction, private_key: &[u8]) -> Result<SignatureBytes, SignerError> {
    let message = transaction.serialize_message().map_err(|e| SignerError::signing_error(format!("serialize message: {e}")))?;
    sign_solana_message(private_key, &message).map_err(|e| SignerError::signing_error(format!("sign: {e}")))
}

pub(super) fn encode_transaction(transaction: &VersionedTransaction) -> Result<String, SignerError> {
    let bytes = transaction.serialize().map_err(|e| SignerError::signing_error(format!("serialize transaction: {e}")))?;
    Ok(encode_base64(&bytes))
}

pub(super) fn sign_and_encode_transaction(mut transaction: VersionedTransaction, private_key: &[u8]) -> Result<String, SignerError> {
    let signature = sign_transaction(&transaction, private_key)?;
    let signatures = transaction.signatures_mut();
    if let Some(first_signature) = signatures.first_mut() {
        *first_signature = signature;
    } else {
        signatures.push(signature);
    }
    encode_transaction(&transaction)
}

pub(crate) fn prepare_with_fee(transaction_base64: &str, fee: &TransactionFee) -> Result<VersionedTransaction, SignerError> {
    let mut transaction = decode_transaction(transaction_base64).map_err(SignerError::invalid_input)?;

    if transaction.signatures().len() > 1 {
        return Ok(transaction);
    }

    let (unit_price, compute_unit_limit) = compute_budget_values(fee)?;
    if unit_price > 0 && !transaction.set_compute_unit_price(unit_price).map_err(SignerError::from_display)? {
        insert_compute_budget_instruction(&mut transaction, set_compute_unit_price(unit_price))?;
    }
    if compute_unit_limit > 0 && !transaction.set_compute_unit_limit(compute_unit_limit).map_err(SignerError::from_display)? {
        insert_compute_budget_instruction(&mut transaction, set_compute_unit_limit(compute_unit_limit))?;
    }

    Ok(transaction)
}

fn compute_budget_values(fee: &TransactionFee) -> Result<(u64, u32), SignerError> {
    let unit_price = fee.unit_price_u64()?;
    let compute_unit_limit = fee.gas_limit.to_u32().ok_or_else(|| SignerError::invalid_input("invalid compute unit limit"))?;
    Ok((unit_price, compute_unit_limit))
}

fn insert_compute_budget_instruction(transaction: &mut VersionedTransaction, instruction: Instruction) -> Result<(), SignerError> {
    match transaction {
        VersionedTransaction::Legacy { message, .. } => insert_compiled_instruction(&mut message.header, &mut message.account_keys, &mut message.instructions, instruction, false),
        VersionedTransaction::V0 { message, .. } => insert_compiled_instruction(&mut message.header, &mut message.account_keys, &mut message.instructions, instruction, true),
    }
}

fn insert_compiled_instruction(
    header: &mut MessageHeader,
    account_keys: &mut Vec<Pubkey>,
    instructions: &mut Vec<CompiledInstruction>,
    instruction: Instruction,
    has_loaded_addresses: bool,
) -> Result<(), SignerError> {
    if !instruction.accounts.is_empty() {
        return SignerError::invalid_input_err("compute budget instruction must not have accounts");
    }
    let insert_position = instructions
        .first()
        .filter(|compiled| account_keys.get(compiled.program_id_index as usize) == Some(&system_program()) && compiled.data.get(0..4) == Some(&4u32.to_le_bytes()))
        .map(|_| 1)
        .unwrap_or(0);
    let program_id_index = match account_keys.iter().position(|pubkey| *pubkey == instruction.program_id) {
        Some(index) => u8::try_from(index).map_err(|_| SignerError::invalid_input("Solana transaction has too many account keys"))?,
        None => {
            let index = u8::try_from(account_keys.len()).map_err(|_| SignerError::invalid_input("Solana transaction has too many account keys"))?;
            if has_loaded_addresses {
                for compiled in instructions.iter_mut() {
                    shift_loaded_index(&mut compiled.program_id_index, index)?;
                    for account in &mut compiled.accounts {
                        shift_loaded_index(account, index)?;
                    }
                }
            }
            header.num_readonly_unsigned_accounts = header
                .num_readonly_unsigned_accounts
                .checked_add(1)
                .ok_or_else(|| SignerError::invalid_input("Solana transaction has too many readonly accounts"))?;
            account_keys.push(instruction.program_id);
            index
        }
    };
    instructions.insert(
        insert_position,
        CompiledInstruction {
            program_id_index,
            accounts: vec![],
            data: instruction.data,
        },
    );
    Ok(())
}

fn shift_loaded_index(index: &mut u8, static_account_count: u8) -> Result<(), SignerError> {
    if *index >= static_account_count {
        *index = index
            .checked_add(1)
            .ok_or_else(|| SignerError::invalid_input("Solana transaction has too many account keys"))?;
    }
    Ok(())
}

pub(super) fn build_legacy_transaction(fee_payer: Pubkey, recent_blockhash: [u8; 32], instructions: Vec<Instruction>) -> Result<VersionedTransaction, SignerError> {
    let mut flags = HashMap::new();
    let mut account_order = Vec::new();
    let mut program_order = Vec::new();

    merge_account(&mut flags, &mut account_order, fee_payer, true, true);
    for instruction in &instructions {
        for account in &instruction.accounts {
            merge_account(&mut flags, &mut account_order, account.pubkey, account.is_signer, account.is_writable);
        }
        merge_account(&mut flags, &mut program_order, instruction.program_id, false, false);
    }

    let mut writable_signers = Vec::new();
    let mut readonly_signers = Vec::new();
    let mut writable_non_signers = Vec::new();
    let mut readonly_non_signers = Vec::new();
    for pubkey in account_order.iter().chain(program_order.iter()) {
        let flags = flags.get(pubkey).ok_or_else(|| SignerError::invalid_input("missing Solana account flags"))?;
        match (flags.is_signer, flags.is_writable) {
            (true, true) => writable_signers.push(*pubkey),
            (true, false) => readonly_signers.push(*pubkey),
            (false, true) => writable_non_signers.push(*pubkey),
            (false, false) => readonly_non_signers.push(*pubkey),
        }
    }

    let num_required_signatures = writable_signers.len() + readonly_signers.len();
    let num_readonly_signed_accounts = readonly_signers.len();
    let num_readonly_unsigned_accounts = readonly_non_signers.len();
    let mut account_keys = Vec::with_capacity(account_order.len() + program_order.len());
    account_keys.push(fee_payer);
    account_keys.extend(writable_signers.into_iter().filter(|pubkey| *pubkey != fee_payer));
    account_keys.extend(readonly_signers);
    account_keys.extend(writable_non_signers);
    account_keys.extend(readonly_non_signers);
    if account_keys.len() > u8::MAX as usize || num_required_signatures > u8::MAX as usize {
        return Err(SignerError::invalid_input("Solana transaction has too many account keys"));
    }

    let key_to_index = account_keys.iter().enumerate().map(|(index, pubkey)| (*pubkey, index as u8)).collect::<HashMap<_, _>>();
    let compiled_instructions = instructions
        .iter()
        .map(|instruction| {
            let program_id_index = account_index(&key_to_index, instruction.program_id)?;
            let accounts = instruction
                .accounts
                .iter()
                .map(|account| account_index(&key_to_index, account.pubkey))
                .collect::<Result<Vec<_>, SignerError>>()?;
            Ok(CompiledInstruction {
                program_id_index,
                accounts,
                data: instruction.data.clone(),
            })
        })
        .collect::<Result<Vec<_>, SignerError>>()?;

    let header = MessageHeader {
        num_required_signatures: num_required_signatures as u8,
        num_readonly_signed_accounts: num_readonly_signed_accounts as u8,
        num_readonly_unsigned_accounts: num_readonly_unsigned_accounts as u8,
    };
    Ok(VersionedTransaction::Legacy {
        signatures: vec![SignatureBytes::new([0u8; 64]); num_required_signatures],
        message: LegacyMessage {
            header,
            account_keys,
            recent_blockhash,
            instructions: compiled_instructions,
        },
    })
}

fn merge_account(flags: &mut HashMap<Pubkey, AccountFlags>, order: &mut Vec<Pubkey>, pubkey: Pubkey, is_signer: bool, is_writable: bool) {
    flags
        .entry(pubkey)
        .and_modify(|flags| {
            flags.is_signer |= is_signer;
            flags.is_writable |= is_writable;
        })
        .or_insert_with(|| {
            order.push(pubkey);
            AccountFlags { is_signer, is_writable }
        });
}

fn account_index(key_to_index: &HashMap<Pubkey, u8>, pubkey: Pubkey) -> Result<u8, SignerError> {
    key_to_index.get(&pubkey).copied().ok_or_else(|| SignerError::invalid_input("missing Solana account key"))
}

pub(super) fn block_hash(input: &SignerInput) -> Result<[u8; 32], SignerError> {
    let block_hash = input.metadata.get_block_hash()?;
    let bytes = bs58::decode(&block_hash).into_vec().map_err(|_| SignerError::invalid_input("invalid Solana block hash"))?;
    bytes.try_into().map_err(|_| SignerError::invalid_input("Solana block hash must be 32 bytes"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{decode_transaction, signer::testkit::SINGLE_SIG_TX};
    use primitives::{AssetId, Chain, GasPriceType};
    use solana_primitives::{AccountMeta, MessageAddressTableLookup, VersionedMessageV0};

    fn pubkey(value: u8) -> Pubkey {
        Pubkey::new([value; 32])
    }

    #[test]
    fn test_decode_transaction_compute_unit_limit() {
        let transaction = decode_transaction(SINGLE_SIG_TX).unwrap();

        assert_eq!(transaction.get_compute_unit_limit(), Some(1_400_000));
    }

    #[test]
    fn test_prepare_with_fee_inserts_compute_budget_for_v0() {
        let mut transaction = VersionedTransaction::V0 {
            signatures: vec![SignatureBytes::new([0; 64])],
            message: VersionedMessageV0 {
                header: MessageHeader {
                    num_required_signatures: 1,
                    num_readonly_signed_accounts: 0,
                    num_readonly_unsigned_accounts: 1,
                },
                account_keys: vec![pubkey(1), system_program()],
                recent_blockhash: [3; 32],
                instructions: vec![CompiledInstruction {
                    program_id_index: 1,
                    accounts: vec![2],
                    data: 4u32.to_le_bytes().to_vec(),
                }],
                address_table_lookups: vec![MessageAddressTableLookup::new(pubkey(5), vec![0], vec![])],
            },
        };
        let encoded = encode_base64(&transaction.serialize().unwrap());
        let fee = TransactionFee::new_gas_price_type(
            GasPriceType::solana(5_000u64, 0u64, 25_000u64),
            5_000u64.into(),
            85_002u64.into(),
            Default::default(),
            AssetId::from_chain(Chain::Solana),
        );

        transaction = prepare_with_fee(&encoded, &fee).unwrap();

        assert_eq!(transaction.get_compute_unit_limit(), Some(85_002));
        assert_eq!(transaction.get_compute_unit_price(), Some(25_000));
        assert_eq!(transaction.instructions()[0].program_id_index, 1);
        assert_eq!(transaction.instructions()[0].accounts, vec![3]);
    }

    #[test]
    fn test_build_legacy_transaction_preserves_account_order_by_bucket() {
        let fee_payer = pubkey(1);
        let writable = pubkey(2);
        let readonly_first = pubkey(3);
        let readonly_second = pubkey(4);
        let program_first = pubkey(5);
        let program_second = pubkey(6);
        let instructions = vec![
            Instruction {
                program_id: program_first,
                accounts: vec![
                    AccountMeta {
                        pubkey: fee_payer,
                        is_signer: true,
                        is_writable: false,
                    },
                    AccountMeta {
                        pubkey: readonly_first,
                        is_signer: false,
                        is_writable: false,
                    },
                    AccountMeta {
                        pubkey: writable,
                        is_signer: false,
                        is_writable: true,
                    },
                ],
                data: vec![1],
            },
            Instruction {
                program_id: program_second,
                accounts: vec![AccountMeta {
                    pubkey: readonly_second,
                    is_signer: false,
                    is_writable: false,
                }],
                data: vec![2],
            },
        ];

        let transaction = build_legacy_transaction(fee_payer, [0; 32], instructions).unwrap();

        assert_eq!(
            transaction.account_keys(),
            &[fee_payer, writable, readonly_first, readonly_second, program_first, program_second]
        );
        assert_eq!(transaction.num_required_signatures(), 1);
        assert_eq!(transaction.num_readonly_unsigned_accounts(), 4);
    }

    #[test]
    fn test_build_legacy_transaction_upgrades_duplicate_account_flags() {
        let fee_payer = pubkey(1);
        let upgraded = pubkey(2);
        let program = pubkey(3);
        let instructions = vec![
            Instruction {
                program_id: program,
                accounts: vec![AccountMeta {
                    pubkey: upgraded,
                    is_signer: false,
                    is_writable: false,
                }],
                data: vec![1],
            },
            Instruction {
                program_id: program,
                accounts: vec![AccountMeta {
                    pubkey: upgraded,
                    is_signer: false,
                    is_writable: true,
                }],
                data: vec![2],
            },
        ];

        let transaction = build_legacy_transaction(fee_payer, [0; 32], instructions).unwrap();

        assert_eq!(transaction.account_keys(), &[fee_payer, upgraded, program]);
        assert_eq!(transaction.num_readonly_unsigned_accounts(), 1);
    }
}
