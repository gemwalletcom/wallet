use super::{instructions, swap, transaction};
use crate::{VersionedTransactionExt, decode_transaction, transaction::is_transaction_bytes};
use gem_encoding::encode_base64;
use primitives::{ApplicationMetadataSource, ChainSigner, SignerError, SignerInput, TransferDataOutputType};
use solana_primitives::{Pubkey, sign_message as sign_solana_message};

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
        Ok(vec![transaction::sign_single_signer_instructions(
            input,
            private_key,
            sender,
            instructions::stake(input, sender)?,
        )?])
    }

    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        if is_transaction_bytes(message) {
            return Err(SignerError::invalid_input(SIGN_MESSAGE_PAYLOAD_REJECTION));
        }
        let signature = sign_solana_message(private_key, message).map_err(|e| SignerError::signing_error(format!("sign: {e}")))?;
        Ok(bs58::encode(signature.as_bytes()).into_string())
    }

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let extra = input.input_type.get_generic_data().map_err(SignerError::invalid_input)?;
        let metadata = input.input_type.get_application_metadata().map_err(SignerError::invalid_input)?;
        let data = extra.data_as_str().map_err(SignerError::invalid_input)?;
        let mut transaction = decode_transaction(data).map_err(SignerError::invalid_input)?;

        let signatures = transaction.signatures();
        if signatures.is_empty() || signatures[0].as_bytes() != &[0u8; 64] {
            return Err(SignerError::invalid_input("user signature should be first"));
        }

        if metadata.source == ApplicationMetadataSource::Payment {
            *transaction.recent_blockhash_mut() = transaction::block_hash(input)?;
        }

        let message_bytes = transaction.serialize_message().map_err(|e| SignerError::signing_error(format!("serialize message: {e}")))?;
        let signature = sign_solana_message(private_key, &message_bytes).map_err(|e| SignerError::signing_error(format!("sign: {e}")))?;

        match extra.output_type {
            TransferDataOutputType::Signature => Ok(bs58::encode(signature.as_bytes()).into_string()),
            TransferDataOutputType::EncodedTransaction => {
                transaction.signatures_mut()[0] = signature;
                let bytes = transaction.serialize().map_err(|e| SignerError::signing_error(format!("serialize transaction: {e}")))?;
                Ok(encode_base64(&bytes))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::testkit::{DOUBLE_SIG_TX, EXPECTED_MESSAGE_HEX, SINGLE_SIG_TX, mock_legacy_transaction};
    use gem_encoding::decode_base64;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{ApplicationMetadataSource, Chain, ChainSigner, SignerInput, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, TransferDataOutputType};
    use solana_primitives::VersionedTransaction;

    #[test]
    fn test_deserialize_single_signature_transaction() {
        let bytes = decode_base64(SINGLE_SIG_TX).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&bytes).unwrap();

        assert_eq!(transaction.signatures().len(), 1);

        let message_bytes = transaction.serialize_message().unwrap();
        let message_hex: String = message_bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(message_hex, EXPECTED_MESSAGE_HEX);
    }

    #[test]
    fn test_deserialize_double_signature_transaction() {
        let bytes = decode_base64(DOUBLE_SIG_TX).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&bytes).unwrap();

        assert_eq!(transaction.signatures().len(), 2);
    }

    #[test]
    fn test_sign_data_encoded_transaction() {
        let signer = SolanaChainSigner;
        let input = TransactionLoadInput::mock_sign_data(Chain::Solana, SINGLE_SIG_TX, TransferDataOutputType::EncodedTransaction);
        let fee = input.default_fee();
        let input = SignerInput::new(input, fee);

        let result = signer.sign_data(&input, &TEST_PRIVATE_KEY).unwrap();

        let signed_bytes = decode_base64(&result).unwrap();
        let signed_transaction = VersionedTransaction::deserialize_with_version(&signed_bytes).unwrap();
        assert_eq!(signed_transaction.signatures().len(), 1);
        assert_ne!(signed_transaction.signatures()[0].as_bytes(), &[0u8; 64]);
    }

    #[test]
    fn test_sign_data_uses_latest_blockhash_for_payment() {
        let mut transaction = mock_legacy_transaction();
        *transaction.recent_blockhash_mut() = [7; 32];
        transaction.add_signature(solana_primitives::SignatureBytes::new([0; 64]));
        let encoded = encode_base64(&transaction.serialize().unwrap());
        let blockhash = bs58::encode([4; 32]).into_string();
        let mut input = TransactionLoadInput::mock_sign_data(Chain::Solana, &encoded, TransferDataOutputType::EncodedTransaction);
        let TransactionInputType::Generic(_, metadata, _) = &mut input.input_type else {
            panic!("expected generic transaction input");
        };
        metadata.source = ApplicationMetadataSource::Payment;
        input.metadata = TransactionLoadMetadata::mock_solana(&blockhash);
        let fee = input.default_fee();
        let input = SignerInput::new(input, fee);

        let result = SolanaChainSigner.sign_data(&input, &TEST_PRIVATE_KEY).unwrap();
        let signed = VersionedTransaction::deserialize_with_version(&decode_base64(&result).unwrap()).unwrap();
        assert_eq!(signed.recent_blockhash(), &[4; 32]);
    }

    #[test]
    fn test_sign_data_preserves_wallet_connect_blockhash() {
        let mut transaction = mock_legacy_transaction();
        *transaction.recent_blockhash_mut() = [0; 32];
        transaction.add_signature(solana_primitives::SignatureBytes::new([0; 64]));
        let encoded = encode_base64(&transaction.serialize().unwrap());
        let mut input = TransactionLoadInput::mock_sign_data(Chain::Solana, &encoded, TransferDataOutputType::EncodedTransaction);
        input.metadata = TransactionLoadMetadata::mock_solana(&bs58::encode([4; 32]).into_string());
        let fee = input.default_fee();

        let result = SolanaChainSigner.sign_data(&SignerInput::new(input, fee), &TEST_PRIVATE_KEY).unwrap();
        let signed = VersionedTransaction::deserialize_with_version(&decode_base64(&result).unwrap()).unwrap();
        assert_eq!(signed.recent_blockhash(), &[0; 32]);
    }

    #[test]
    fn test_sign_data_signature_output() {
        let signer = SolanaChainSigner;
        let input = TransactionLoadInput::mock_sign_data(Chain::Solana, SINGLE_SIG_TX, TransferDataOutputType::Signature);
        let fee = input.default_fee();
        let input = SignerInput::new(input, fee);

        let result = signer.sign_data(&input, &TEST_PRIVATE_KEY).unwrap();

        let sig_bytes = bs58::decode(&result).into_vec().unwrap();
        assert_eq!(sig_bytes.len(), 64);
    }

    #[test]
    fn test_sign_message() {
        let result = SolanaChainSigner.sign_message(b"hello", &TEST_PRIVATE_KEY).unwrap();

        assert_eq!(bs58::decode(result).into_vec().unwrap().len(), 64);
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
    }
}
