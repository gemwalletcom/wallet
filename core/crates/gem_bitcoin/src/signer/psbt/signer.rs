use bitcoin::{
    Psbt, PublicKey, ScriptBuf, Witness,
    consensus::serialize,
    secp256k1::{Message, PublicKey as Secp256k1PublicKey, Secp256k1, SecretKey},
    sighash::{EcdsaSighashType, SighashCache},
};
use primitives::{BitcoinChain, SignerError};

use crate::signer::{
    address::script_for_address,
    transaction::{ZeroizedSecretKey, der_signature},
};

pub fn sign_transaction(psbt: &Psbt, sender: &str, private_key: &[u8]) -> Result<String, SignerError> {
    let secret_key = ZeroizedSecretKey(SecretKey::from_slice(private_key).map_err(|_| SignerError::invalid_input("invalid Bitcoin private key"))?);
    let secp = Secp256k1::new();
    let public_key = PublicKey::new(Secp256k1PublicKey::from_secret_key(&secp, &secret_key.0));
    let sender_script = script_for_address(BitcoinChain::Bitcoin, sender)?.script_pubkey;
    let expected_script = ScriptBuf::new_p2wpkh(&public_key.wpubkey_hash().map_err(SignerError::from_display)?);
    if sender_script != expected_script {
        return SignerError::invalid_input_err("Bitcoin private key does not match sender address");
    }
    let previous_outputs = psbt.iter_funding_utxos().collect::<Result<Vec<_>, _>>().map_err(SignerError::from_display)?;
    let mut transaction = psbt.unsigned_tx.clone();
    let mut cache = SighashCache::new(&psbt.unsigned_tx);
    for (index, transaction_input) in transaction.input.iter_mut().enumerate() {
        let sighash = cache
            .p2wpkh_signature_hash(index, &sender_script, previous_outputs[index].value, EcdsaSighashType::All)
            .map_err(SignerError::from_display)?;
        transaction_input.witness = Witness::from_slice(&[
            der_signature(&secp, &secret_key.0, Message::from(sighash), EcdsaSighashType::All.to_u32() as u8),
            public_key.to_bytes(),
        ]);
    }
    Ok(hex::encode(serialize(&transaction)))
}
