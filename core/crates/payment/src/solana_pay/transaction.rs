use gem_encoding::encode_base64;
use gem_solana::{VersionedTransactionExt, decode_transaction};
use primitives::{PaymentAmount, PaymentRequest, TransactionType};
use solana_primitives::{Pubkey, SignatureBytes};

pub(super) struct PreparedTransaction {
    pub transaction: String,
    pub transaction_type: TransactionType,
    pub memo: Option<String>,
    pub request: Option<PaymentRequest>,
}

pub(super) fn prepare(transaction: &str, signer: &str) -> Result<PreparedTransaction, String> {
    let mut transaction = decode_transaction(transaction).map_err(|_| "failed to decode Solana Pay transaction".to_string())?;

    let signer = Pubkey::from_base58(signer).map_err(|_| "invalid Solana Pay signer".to_string())?;
    if transaction.num_required_signatures() != 1 {
        return Err("Solana Pay transaction must require exactly one signer".to_string());
    }

    let fee_payer = transaction
        .account_keys_mut()
        .first_mut()
        .ok_or_else(|| "Solana Pay transaction has no fee payer".to_string())?;
    let zero = Pubkey::new([0u8; 32]);
    if *fee_payer != signer && *fee_payer != zero {
        return Err("Solana Pay transaction fee payer does not match the wallet account".to_string());
    }
    *fee_payer = signer;

    match transaction.signatures() {
        [] => transaction.add_signature(SignatureBytes::new([0u8; 64])),
        [signature] if signature.as_bytes() == &[0u8; 64] => {}
        [_] => return Err("Solana Pay transaction already contains the wallet signature".to_string()),
        _ => return Err("Solana Pay transaction has an invalid signature count".to_string()),
    }
    let transaction_type = transaction.transaction_type();
    let memo = transaction.memo();
    Ok(PreparedTransaction {
        request: transaction.simple_transfer(&signer).map(|transfer| PaymentRequest {
            address: transfer.recipient,
            amount: Some(PaymentAmount::AtomicValue(transfer.value)),
            memo: memo.clone(),
            asset_id: Some(transfer.asset_id),
        }),
        transaction_type,
        memo,
        transaction: transaction
            .serialize()
            .map(|bytes| encode_base64(&bytes))
            .map_err(|error| format!("failed to serialize Solana Pay transaction: {error}"))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::decode_base64;
    use gem_solana::MEMO_PROGRAM_ID;
    use solana_primitives::{CompiledInstruction, VersionedTransaction};

    #[test]
    fn test_prepare() {
        const ACCOUNT: &str = "mvines9iiHiQTysrwkJjGf2gb9Ex9jXJX8ns3qwf2kN";
        const TRANSACTION: &str = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAECC4JMKqNplIXybGb/GhK1ofdVWeuEjXnQor7gi0Y2hMcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQECAAAMAgAAAAAAAAAAAAAA";

        let mut requested = VersionedTransaction::deserialize_with_version(&decode_base64(TRANSACTION).unwrap()).unwrap();
        *requested.recent_blockhash_mut() = [7; 32];
        let memo = "ck:262:operator:m:1787598390";
        match &mut requested {
            VersionedTransaction::Legacy { message, .. } => {
                let program_id_index = message.account_keys.len() as u8;
                message.account_keys.push(Pubkey::from_base58(MEMO_PROGRAM_ID).unwrap());
                message.header.num_readonly_unsigned_accounts += 1;
                message.instructions.push(CompiledInstruction {
                    program_id_index,
                    accounts: vec![],
                    data: memo.as_bytes().to_vec(),
                });
            }
            VersionedTransaction::V0 { message, .. } => {
                let program_id_index = message.account_keys.len() as u8;
                message.account_keys.push(Pubkey::from_base58(MEMO_PROGRAM_ID).unwrap());
                message.header.num_readonly_unsigned_accounts += 1;
                message.instructions.push(CompiledInstruction {
                    program_id_index,
                    accounts: vec![],
                    data: memo.as_bytes().to_vec(),
                });
            }
        }
        let requested = encode_base64(&requested.serialize().unwrap());

        let prepared = prepare(&requested, ACCOUNT).unwrap();
        let bytes = decode_base64(&prepared.transaction).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&bytes).unwrap();

        assert_eq!(transaction.num_required_signatures(), 1);
        assert_eq!(transaction.account_keys()[0].to_base58(), ACCOUNT);
        assert_eq!(transaction.signatures().len(), 1);
        assert_eq!(transaction.signatures()[0].as_bytes(), &[0u8; 64]);
        assert_eq!(transaction.recent_blockhash(), &[7u8; 32]);
        assert_eq!(prepared.transaction_type, TransactionType::Transfer);
        assert_eq!(prepared.memo.as_deref(), Some(memo));
    }
}
