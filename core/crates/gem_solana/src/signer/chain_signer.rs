use std::str;

use ::signer::Ed25519KeyPair;
use chrono::Utc;
use gem_encoding::encode_base64;
use primitives::{Chain, ChainSigner, SignerError, SignerInput, TransactionInputType, TransferDataOutputType};

use super::{instructions, sign_message as sign_solana_message, swap, transaction};
use crate::{Pubkey, VersionedTransactionExt, decode_transaction, siws::SiwsMessage, transaction::is_transaction_bytes};

#[derive(Default)]
pub struct SolanaChainSigner;

const SIGN_MESSAGE_PAYLOAD_REJECTION: &str = "Serialized Solana transaction or transaction message received in signMessage request; use signTransaction instead";

impl ChainSigner for SolanaChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let sender = Pubkey::from_base58(&input.sender_address).map_err(SignerError::from_display)?;
        transaction::sign_single_signer_instructions(input, private_key, sender, instructions::native_transfer(input, sender)?)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let sender = Pubkey::from_base58(&input.sender_address).map_err(SignerError::from_display)?;
        transaction::sign_single_signer_instructions(input, private_key, sender, instructions::token_transfer(input, sender)?)
    }

    fn sign_nft_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let sender = Pubkey::from_base58(&input.sender_address).map_err(SignerError::from_display)?;
        transaction::sign_single_signer_instructions(input, private_key, sender, instructions::nft_transfer(input, sender)?)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        swap::sign(input, private_key)
    }

    fn sign_stake(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let sender = Pubkey::from_base58(&input.sender_address).map_err(SignerError::from_display)?;
        Ok(vec![transaction::sign_single_signer_instructions(input, private_key, sender, instructions::stake(input, sender)?)?])
    }

    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        if is_transaction_bytes(message) {
            return Err(SignerError::invalid_input(SIGN_MESSAGE_PAYLOAD_REJECTION));
        }
        if let Ok(raw) = str::from_utf8(message)
            && let Some(siws) = SiwsMessage::parse(raw).map_err(SignerError::invalid_input)?
        {
            siws.validate(Chain::Solana, Utc::now()).map_err(SignerError::invalid_input)?;
            let public_key = Ed25519KeyPair::from_private_key(private_key)?.public_key_bytes;
            if siws.address != bs58::encode(public_key).into_string() {
                return SignerError::invalid_input_err("SIWS account mismatch");
            }
        }
        let signature = sign_solana_message(private_key, message).map_err(|e| SignerError::signing_error(format!("sign: {e}")))?;
        Ok(bs58::encode(signature.as_bytes()).into_string())
    }

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let extra = input.input_type.get_generic_data().map_err(SignerError::invalid_input)?;
        let data = extra.data_as_str().map_err(SignerError::invalid_input)?;
        let mut transaction = decode_transaction(data).map_err(SignerError::invalid_input)?;

        if let TransactionInputType::Payment { .. } = input.input_type
            && !transaction.uses_durable_nonce()
        {
            *transaction.recent_blockhash_mut() = transaction::block_hash(input)?;
        }

        let signature = transaction::sign_transaction(&mut transaction, private_key)?;

        match extra.output_type {
            TransferDataOutputType::Signature => Ok(bs58::encode(signature.as_bytes()).into_string()),
            TransferDataOutputType::EncodedTransaction => {
                let bytes = transaction.serialize().map_err(|e| SignerError::signing_error(format!("serialize transaction: {e}")))?;
                Ok(encode_base64(&bytes))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::{program_ids::system_program, system::ADVANCE_NONCE_ACCOUNT_DISCRIMINANT};
    use crate::signer::testkit::SINGLE_SIG_TX;
    use crate::testkit::mock_legacy_transaction;
    use crate::testkit::mock_v1_transaction;
    use crate::{CompiledInstruction, SignatureBytes, VersionedTransaction};
    use gem_encoding::decode_base64;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{Chain, ChainSigner, SignerInput, TransactionLoadInput, TransactionLoadMetadata, TransferDataOutputType};

    fn signed_blockhash(transaction: VersionedTransaction, payment: bool) -> [u8; 32] {
        let encoded = encode_base64(&transaction.serialize().unwrap());
        let mut input = if payment {
            TransactionLoadInput::mock_sign_data_payment(Chain::Solana, &encoded, TransferDataOutputType::EncodedTransaction)
        } else {
            TransactionLoadInput::mock_sign_data(Chain::Solana, &encoded, TransferDataOutputType::EncodedTransaction)
        };
        input.metadata = TransactionLoadMetadata::mock_solana(&bs58::encode([4; 32]).into_string());
        let fee = input.default_fee();

        let result = SolanaChainSigner.sign_data(&SignerInput::new(input, fee), &TEST_PRIVATE_KEY).unwrap();

        *VersionedTransaction::deserialize_with_version(&decode_base64(&result).unwrap()).unwrap().recent_blockhash()
    }

    #[test]
    fn test_sign_data_blockhash() {
        let mut latest_blockhash = mock_legacy_transaction();
        *latest_blockhash.recent_blockhash_mut() = [7; 32];
        latest_blockhash.add_signature(SignatureBytes::new([0; 64]));

        let mut durable_nonce = mock_legacy_transaction();
        *durable_nonce.recent_blockhash_mut() = [7; 32];
        let message = durable_nonce.message_mut();
        message.account_keys.push(system_program());
        message.header.num_readonly_unsigned_accounts += 1;
        message.instructions.insert(0, CompiledInstruction::mock(2, vec![1], ADVANCE_NONCE_ACCOUNT_DISCRIMINANT.to_vec()));
        durable_nonce.add_signature(SignatureBytes::new([0; 64]));

        let mut wallet_connect = mock_legacy_transaction();
        *wallet_connect.recent_blockhash_mut() = [0; 32];
        wallet_connect.add_signature(SignatureBytes::new([0; 64]));

        assert_eq!(signed_blockhash(latest_blockhash, true), [4; 32]);
        assert_eq!(signed_blockhash(durable_nonce, true), [7; 32]);
        assert_eq!(signed_blockhash(wallet_connect, false), [0; 32]);
    }

    #[test]
    fn test_sign_data_fills_the_wallet_signer_slot() {
        let transaction = mock_v1_transaction(2, 1);
        let message_bytes = transaction.serialize_message().unwrap();
        let transaction_config = *transaction.transaction_config().unwrap();
        let fee_payer_signature = transaction.signatures()[0];
        let encoded = encode_base64(&transaction.serialize().unwrap());
        let input = TransactionLoadInput::mock_sign_data(Chain::Solana, &encoded, TransferDataOutputType::EncodedTransaction);
        let fee = input.default_fee();

        let result = SolanaChainSigner.sign_data(&SignerInput::new(input, fee), &TEST_PRIVATE_KEY).unwrap();

        let signed = VersionedTransaction::deserialize_with_version(&decode_base64(&result).unwrap()).unwrap();
        let expected_signature = sign_solana_message(&TEST_PRIVATE_KEY, &message_bytes).unwrap();
        assert_eq!(signed.serialize_message().unwrap(), message_bytes);
        assert_eq!(*signed.transaction_config().unwrap(), transaction_config);
        assert_eq!(signed.signatures(), &[fee_payer_signature, expected_signature]);

        let signature_input = TransactionLoadInput::mock_sign_data(Chain::Solana, &encoded, TransferDataOutputType::Signature);
        let fee = signature_input.default_fee();

        let signature = SolanaChainSigner.sign_data(&SignerInput::new(signature_input, fee), &TEST_PRIVATE_KEY).unwrap();

        assert_eq!(signature, bs58::encode(expected_signature.as_bytes()).into_string());
    }

    #[test]
    fn test_sign_data_rejects_transactions_the_wallet_must_not_sign() {
        let input = TransactionLoadInput::mock_sign_data(Chain::Solana, SINGLE_SIG_TX, TransferDataOutputType::EncodedTransaction);
        let fee = input.default_fee();

        assert_eq!(
            SolanaChainSigner.sign_data(&SignerInput::new(input, fee), &TEST_PRIVATE_KEY).unwrap_err().to_string(),
            "Invalid input: wallet account is not a required signer of the Solana transaction"
        );

        let mut transaction = mock_v1_transaction(1, 0);
        transaction.signatures_mut()[0] = SignatureBytes::new([9; 64]);
        let encoded = encode_base64(&transaction.serialize().unwrap());
        let input = TransactionLoadInput::mock_sign_data(Chain::Solana, &encoded, TransferDataOutputType::EncodedTransaction);
        let fee = input.default_fee();

        assert_eq!(
            SolanaChainSigner.sign_data(&SignerInput::new(input, fee), &TEST_PRIVATE_KEY).unwrap_err().to_string(),
            "Invalid input: Solana transaction already contains the wallet signature"
        );
    }

    #[test]
    fn test_sign_message() {
        let result = SolanaChainSigner.sign_message(b"hello", &TEST_PRIVATE_KEY).unwrap();

        assert_eq!(bs58::decode(result).into_vec().unwrap().len(), 64);
    }

    #[test]
    fn test_sign_message_siws() {
        let message = include_str!("../../testdata/siws_sign_in.txt");
        let signature = SolanaChainSigner.sign_message(message.as_bytes(), &TEST_PRIVATE_KEY).unwrap();
        assert_eq!(signature, bs58::encode(sign_solana_message(&TEST_PRIVATE_KEY, message.as_bytes()).unwrap().as_bytes()).into_string());
        assert_eq!(SolanaChainSigner.sign_message(message.as_bytes(), &[2; 32]).unwrap_err().to_string(), "Invalid input: SIWS account mismatch");
        let expired_message = include_str!("../../testdata/siws_complete.txt");
        assert_eq!(
            SolanaChainSigner.sign_message(expired_message.as_bytes(), &TEST_PRIVATE_KEY).unwrap_err().to_string(),
            "Invalid input: SIWS message expired or invalid expiration"
        );
        assert_eq!(bs58::decode(SolanaChainSigner.sign_message(&[255, 254, 253], &TEST_PRIVATE_KEY).unwrap()).into_vec().unwrap().len(), 64);
    }

    #[test]
    fn test_sign_message_rejects_transaction_payloads() {
        let bytes = decode_base64(SINGLE_SIG_TX).unwrap();
        let result = SolanaChainSigner.sign_message(&bytes, &TEST_PRIVATE_KEY);

        assert_eq!(result.unwrap_err().to_string(), format!("Invalid input: {SIGN_MESSAGE_PAYLOAD_REJECTION}"));

        let transaction = VersionedTransaction::deserialize_with_version(&bytes).unwrap();
        let message = transaction.serialize_message().unwrap();
        let result = SolanaChainSigner.sign_message(&message, &TEST_PRIVATE_KEY);

        assert_eq!(result.unwrap_err().to_string(), format!("Invalid input: {SIGN_MESSAGE_PAYLOAD_REJECTION}"));

        let message = mock_legacy_transaction().serialize_message().unwrap();
        let result = SolanaChainSigner.sign_message(&message, &TEST_PRIVATE_KEY);

        assert_eq!(result.unwrap_err().to_string(), format!("Invalid input: {SIGN_MESSAGE_PAYLOAD_REJECTION}"));

        let transaction = mock_v1_transaction(1, 0);
        let full_transaction = transaction.serialize().unwrap();
        let message = transaction.serialize_message().unwrap();

        assert_eq!(
            SolanaChainSigner.sign_message(&full_transaction, &TEST_PRIVATE_KEY).unwrap_err().to_string(),
            format!("Invalid input: {SIGN_MESSAGE_PAYLOAD_REJECTION}")
        );
        assert_eq!(SolanaChainSigner.sign_message(&message, &TEST_PRIVATE_KEY).unwrap_err().to_string(), format!("Invalid input: {SIGN_MESSAGE_PAYLOAD_REJECTION}"));
    }
}
