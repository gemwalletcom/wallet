use gem_encoding::encode_base64;
use num_traits::ToPrimitive;
use primitives::{SignerError, SignerInput, TransactionFee};

use super::sign_message;
use crate::{
    AccountMeta, Instruction, Pubkey, VersionedTransaction,
    builder::{AccountBuckets, collect_accounts, compile_legacy},
    instructions::compute_budget::{set_compute_unit_limit, set_compute_unit_price},
};

pub(crate) fn compute_budget_instructions(fee: &TransactionFee) -> Result<Vec<Instruction>, SignerError> {
    let unit_price = fee.unit_price_u64()?;
    let gas_limit = fee.gas_limit.to_u32().ok_or_else(|| SignerError::invalid_input("invalid gas limit"))?;
    let mut instructions = Vec::new();
    if unit_price > 0 {
        instructions.push(set_compute_unit_price(unit_price));
    }
    if gas_limit > 0 {
        instructions.push(set_compute_unit_limit(gas_limit));
    }
    Ok(instructions)
}

pub(crate) fn sign_single_signer_instructions(input: &SignerInput, private_key: &[u8], fee_payer: Pubkey, instructions: Vec<Instruction>) -> Result<String, SignerError> {
    let mut transaction = build_legacy_transaction(fee_payer, block_hash(input)?, &instructions)?;
    if transaction.num_required_signatures() != 1 {
        return Err(SignerError::invalid_input("Solana transaction requires more than one signer"));
    }
    let signature = sign_message(private_key, &transaction.serialize_message()?)?;
    let signature_slot = transaction.signatures_mut().first_mut().ok_or_else(|| SignerError::signing_error("missing Solana signature slot"))?;
    *signature_slot = signature;
    Ok(encode_base64(&transaction.serialize()?))
}

fn build_legacy_transaction(fee_payer: Pubkey, recent_blockhash: [u8; 32], instructions: &[Instruction]) -> Result<VersionedTransaction, SignerError> {
    let accounts = instructions
        .iter()
        .flat_map(|instruction| instruction.accounts.iter().cloned())
        .chain(instructions.iter().map(|instruction| AccountMeta::new_readonly(instruction.program_id)));
    let account_buckets = AccountBuckets::from_accounts(collect_accounts(fee_payer, accounts));
    Ok(compile_legacy(fee_payer, recent_blockhash, &account_buckets, instructions)?)
}

pub(super) fn block_hash(input: &SignerInput) -> Result<[u8; 32], SignerError> {
    let block_hash = input.metadata.get_block_hash()?;
    let bytes = bs58::decode(&block_hash).into_vec().map_err(|_| SignerError::invalid_input("invalid Solana block hash"))?;
    bytes.try_into().map_err(|_| SignerError::invalid_input("Solana block hash must be 32 bytes"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompiledInstruction, Message, MessageHeader, SignatureBytes, decode_transaction, signer::testkit::SINGLE_SIG_TX};

    #[test]
    fn test_decode_transaction_compute_unit_limit() {
        let transaction = decode_transaction(SINGLE_SIG_TX).unwrap();

        assert_eq!(transaction.get_compute_unit_limit(), Some(1_400_000));
    }

    #[test]
    fn test_build_legacy_transaction_preserves_account_order_by_bucket() {
        let fee_payer = Pubkey::new([1; 32]);
        let writable = Pubkey::new([2; 32]);
        let readonly_first = Pubkey::new([3; 32]);
        let readonly_second = Pubkey::new([4; 32]);
        let program_first = Pubkey::new([5; 32]);
        let program_second = Pubkey::new([6; 32]);
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

        let transaction = build_legacy_transaction(fee_payer, [0; 32], &instructions).unwrap();

        assert_eq!(
            transaction,
            VersionedTransaction::Legacy {
                signatures: vec![SignatureBytes::default()],
                message: Message {
                    header: MessageHeader {
                        num_required_signatures: 1,
                        num_readonly_signed_accounts: 0,
                        num_readonly_unsigned_accounts: 4,
                    },
                    account_keys: vec![fee_payer, writable, readonly_first, readonly_second, program_first, program_second],
                    recent_blockhash: [0; 32],
                    instructions: vec![
                        CompiledInstruction {
                            program_id_index: 4,
                            accounts: vec![0, 2, 1],
                            data: vec![1],
                        },
                        CompiledInstruction {
                            program_id_index: 5,
                            accounts: vec![3],
                            data: vec![2],
                        },
                    ],
                },
            }
        );
    }

    #[test]
    fn test_build_legacy_transaction_upgrades_duplicate_account_flags() {
        let fee_payer = Pubkey::new([1; 32]);
        let upgraded = Pubkey::new([2; 32]);
        let program = Pubkey::new([3; 32]);
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

        let transaction = build_legacy_transaction(fee_payer, [0; 32], &instructions).unwrap();

        assert_eq!(transaction.account_keys(), &[fee_payer, upgraded, program]);
        assert_eq!(transaction.message().header.num_readonly_unsigned_accounts, 1);
    }
}
