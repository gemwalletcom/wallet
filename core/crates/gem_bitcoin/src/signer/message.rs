use bitcoin::{
    Amount, CompressedPublicKey, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
    absolute::LockTime,
    consensus::encode::serialize,
    ecdsa,
    hashes::{Hash, sha256, sha256d},
    opcodes::all::OP_RETURN,
    script::Builder,
    secp256k1::{Message, PublicKey, Secp256k1},
    sighash::{EcdsaSighashType, SighashCache},
    transaction::Version,
};
use primitives::{BitcoinChain, SignerError};

use crate::signer::{encoding::encode_varint, transaction::ZeroizedSecretKey};

const MESSAGE_PREFIX: &[u8] = b"\x18Bitcoin Signed Message:\n";
const BIP322_TAG: &[u8] = b"BIP0322-signed-message";
const P2WPKH_RECOVERY_HEADER: u8 = 39;

pub fn sign_message_ecdsa(message: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let secret_key = ZeroizedSecretKey::new(BitcoinChain::Bitcoin, private_key)?;
    let signature = Secp256k1::signing_only().sign_ecdsa_recoverable(&Message::from_digest(message_hash(message)), &secret_key);
    let (recovery_id, compact) = signature.serialize_compact();
    let header = u8::try_from(recovery_id.to_i32()).map_err(|_| SignerError::signing_error("invalid recovery id"))? + P2WPKH_RECOVERY_HEADER;
    Ok([&[header][..], &compact].concat())
}

pub fn sign_message_bip322(message: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
    let secp = Secp256k1::signing_only();
    let secret_key = ZeroizedSecretKey::new(BitcoinChain::Bitcoin, private_key)?;
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    let script_pubkey = ScriptBuf::new_p2wpkh(&CompressedPublicKey(public_key).wpubkey_hash());
    let to_spend = Transaction {
        version: Version(0),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint { txid: Txid::all_zeros(), vout: u32::MAX },
            script_sig: Builder::new().push_int(0).push_slice(bip322_message_hash(message)).into_script(),
            sequence: Sequence::ZERO,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::ZERO,
            script_pubkey: script_pubkey.clone(),
        }],
    };
    let to_sign = Transaction {
        version: Version(0),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint { txid: to_spend.compute_txid(), vout: 0 },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ZERO,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::ZERO,
            script_pubkey: Builder::new().push_opcode(OP_RETURN).into_script(),
        }],
    };
    let sighash = SighashCache::new(&to_sign)
        .p2wpkh_signature_hash(0, &script_pubkey, Amount::ZERO, EcdsaSighashType::All)
        .map_err(|error| SignerError::signing_error(format!("BIP-322 sighash: {error}")))?;
    let signature = ecdsa::Signature {
        signature: secp.sign_ecdsa_low_r(&Message::from_digest(sighash.to_byte_array()), &secret_key),
        sighash_type: EcdsaSighashType::All,
    };
    Ok(serialize(&Witness::p2wpkh(&signature, &public_key)))
}

pub fn message_hash(message: &[u8]) -> [u8; 32] {
    let data = [MESSAGE_PREFIX, &encode_varint(message.len()), message].concat();
    sha256d::Hash::hash(&data).to_byte_array()
}

fn bip322_message_hash(message: &[u8]) -> [u8; 32] {
    let tag = sha256::Hash::hash(BIP322_TAG).to_byte_array();
    sha256::Hash::hash(&[&tag[..], &tag, message].concat()).to_byte_array()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::{
        PrivateKey,
        secp256k1::ecdsa::{RecoverableSignature, RecoveryId},
    };

    const BIP322_PRIVATE_KEY: &str = "L3VFeEujGtevx9w18HD1fhRbCH67Az2dpCymeRE1SoPK6XQtaN2k";

    fn private_key() -> Vec<u8> {
        PrivateKey::from_wif(BIP322_PRIVATE_KEY).unwrap().inner.secret_bytes().to_vec()
    }

    #[test]
    fn test_bip322_signatures_match_the_bip_test_vectors() {
        for (message, expected) in [
            (
                "",
                "AkcwRAIgM2gBAQqvZX15ZiysmKmQpDrG83avLIT492QBzLnQIxYCIBaTpOaD20qRlEylyxFSeEA2ba9YOixpX8z46TSDtS40ASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI=",
            ),
            (
                "Hello World",
                "AkcwRAIgZRfIY3p7/DoVTty6YZbWS71bc5Vct9p9Fia83eRmw2QCICK/ENGfwLtptFluMGs2KsqoNSk89pO7F29zJLUx9a/sASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI=",
            ),
        ] {
            assert_eq!(sign_message_bip322(message.as_bytes(), &private_key()).unwrap(), gem_encoding::decode_base64(expected).unwrap(), "{message:?}");
        }
    }

    #[test]
    fn test_bip322_message_hash_matches_the_bip() {
        assert_eq!(primitives::hex::encode(bip322_message_hash(b"")), "c90c269c4f8fcbe6880f72a721ddfbf1914268a794cbb21cfafee13770ae19f1");
        assert_eq!(primitives::hex::encode(bip322_message_hash(b"Hello World")), "f0eb03b1a75ac6d9847f55c624a99169b5dccba2a31f5b23bea77ba270de0a7a");
    }

    #[test]
    fn test_ecdsa_signature_recovers_the_signing_key_with_a_native_segwit_header() {
        let signature = sign_message_ecdsa(b"Hello World", &private_key()).unwrap();
        assert_eq!(signature.len(), 65);
        assert!((39..=42).contains(&signature[0]), "BIP-137 native segwit header");

        let recovery_id = RecoveryId::from_i32(i32::from(signature[0] - P2WPKH_RECOVERY_HEADER)).unwrap();
        let recoverable = RecoverableSignature::from_compact(&signature[1..], recovery_id).unwrap();
        let recovered = Secp256k1::new().recover_ecdsa(&Message::from_digest(message_hash(b"Hello World")), &recoverable).unwrap();
        assert_eq!(recovered, PublicKey::from_secret_key(&Secp256k1::new(), &ZeroizedSecretKey::new(BitcoinChain::Bitcoin, &private_key()).unwrap()));
    }

    #[test]
    fn test_message_hash_prefixes_the_length() {
        assert_eq!(message_hash(b"Hello World"), bitcoin::sign_message::signed_msg_hash("Hello World").to_byte_array());
    }
}
