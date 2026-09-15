use bitcoin::{
    Address, Amount, Network, OutPoint, Psbt, PublicKey, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, WPubkeyHash, Witness,
    absolute::LockTime,
    hashes::Hash,
    opcodes::all::OP_RETURN,
    script::{Builder, PushBytesBuf},
    secp256k1::{PublicKey as Secp256k1PublicKey, Secp256k1, SecretKey},
    transaction::Version,
};
use num_bigint::BigInt;
use primitives::{BitcoinChain, SignerInput, SwapProvider, TransactionInputType, TransactionLoadMetadata};

use super::signer_mock::{TEST_PRIVATE_KEY, mock_contract_swap_input, mock_utxo_with};

pub fn mock_psbt(sender_script: ScriptBuf) -> (Psbt, SignerInput) {
    let destination = ScriptBuf::new_p2wpkh(&WPubkeyHash::from_byte_array([2; 20]));
    let sender = Address::from_script(&sender_script, Network::Bitcoin).unwrap().to_string();
    let transaction = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::new(Txid::all_zeros(), 0),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        }],
        output: vec![
            TxOut {
                value: Amount::from_sat(10_000),
                script_pubkey: destination.clone(),
            },
            TxOut {
                value: Amount::from_sat(39_000),
                script_pubkey: sender_script.clone(),
            },
            TxOut {
                value: Amount::ZERO,
                script_pubkey: Builder::new()
                    .push_opcode(OP_RETURN)
                    .push_slice(PushBytesBuf::try_from(format!("0x{}", "12".repeat(32)).into_bytes()).unwrap())
                    .into_script(),
            },
        ],
    };
    let mut psbt = Psbt::from_unsigned_tx(transaction).unwrap();
    psbt.inputs[0].witness_utxo = Some(TxOut {
        value: Amount::from_sat(50_000),
        script_pubkey: sender_script,
    });
    let mut input = mock_contract_swap_input(BitcoinChain::Bitcoin, "", false);
    input.input.sender_address = sender.clone();
    input.fee.fee = BigInt::from(1000);
    input.input.metadata = TransactionLoadMetadata::Bitcoin {
        utxos: vec![mock_utxo_with(&Txid::all_zeros().to_string(), 0, "50000", &sender)],
    };
    if let TransactionInputType::Swap { swap_data, .. } = &mut input.input.input_type {
        swap_data.quote.from_address = sender;
        swap_data.quote.provider_data.provider = SwapProvider::Relay;
        swap_data.data.to = Address::from_script(&destination, Network::Bitcoin).unwrap().to_string();
    }
    (psbt, input)
}

pub fn mock_segwit_psbt() -> (Psbt, SignerInput) {
    let secp = Secp256k1::new();
    let public_key = PublicKey::new(Secp256k1PublicKey::from_secret_key(&secp, &SecretKey::from_slice(&TEST_PRIVATE_KEY).unwrap()));
    mock_psbt(ScriptBuf::new_p2wpkh(&public_key.wpubkey_hash().unwrap()))
}
