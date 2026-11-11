use std::collections::BTreeSet;
use std::str::FromStr;

use bitcoin::{
    Address, Amount, CompressedPublicKey, Network, Psbt, ScriptBuf, TxOut, Witness, ecdsa,
    secp256k1::{Message, PublicKey, Secp256k1},
    sighash::{EcdsaSighashType, SighashCache},
};
use num_bigint::BigUint;
use primitives::{SignerError, UTXO, hex};
use serde::Deserialize;

use crate::signer::message::secret_key;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinPsbtRequest {
    pub account: String,
    pub psbt: String,
    #[serde(default)]
    pub sign_inputs: Vec<BitcoinPsbtSignInput>,
    #[serde(default)]
    pub broadcast: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinPsbtSignInput {
    pub address: String,
    pub index: usize,
    pub sighash_types: Option<Vec<u32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PsbtSummary {
    pub recipient: Option<String>,
    pub value: u64,
    pub fee: u64,
}

impl BitcoinPsbtRequest {
    pub fn parse(json: &str) -> Result<Self, SignerError> {
        serde_json::from_str(json).map_err(|error| SignerError::invalid_input(format!("invalid signPsbt request: {error}")))
    }

    pub fn decode_psbt(&self) -> Result<Psbt, SignerError> {
        Psbt::from_str(&self.psbt).map_err(|error| SignerError::invalid_input(format!("invalid PSBT: {error}")))
    }

    pub fn summary(&self, address: &str) -> Result<PsbtSummary, SignerError> {
        let psbt = self.decode_psbt()?;
        let script = wallet_script(address)?;
        let inputs = self.wallet_inputs(&psbt, address, &script)?;
        let spent = total(inputs.iter().map(|input| input.output.value))?;
        let returned = total(psbt.unsigned_tx.output.iter().filter(|output| output.script_pubkey == script).map(|output| output.value))?;
        let fee = match inputs.len() == psbt.inputs.len() {
            true => spent
                .checked_sub(total(psbt.unsigned_tx.output.iter().map(|output| output.value))?)
                .ok_or_else(|| SignerError::invalid_input("PSBT pays out more than its inputs hold"))?,
            false => 0,
        };
        Ok(PsbtSummary {
            recipient: recipient(&psbt, &script)?,
            value: spent.saturating_sub(returned).saturating_sub(fee),
            fee,
        })
    }

    pub fn check_coins(&self, address: &str, utxos: &[UTXO]) -> Result<(), SignerError> {
        let psbt = self.decode_psbt()?;
        let inputs = self.wallet_inputs(&psbt, address, &wallet_script(address)?)?;
        held_coins(&psbt, &inputs, utxos)
    }

    pub fn sign(&self, address: &str, private_key: &[u8], utxos: &[UTXO]) -> Result<Psbt, SignerError> {
        let mut psbt = self.decode_psbt()?;
        let script = wallet_script(address)?;
        let inputs = self.wallet_inputs(&psbt, address, &script)?;
        held_coins(&psbt, &inputs, utxos)?;
        let secp = Secp256k1::signing_only();
        let secret_key = secret_key(private_key)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        if ScriptBuf::new_p2wpkh(&CompressedPublicKey(public_key).wpubkey_hash()) != script {
            return SignerError::invalid_input_err("the wallet key does not own the session address");
        }
        for input in inputs {
            let sighash = SighashCache::new(&psbt.unsigned_tx)
                .p2wpkh_signature_hash(input.index, &script, input.output.value, input.sighash_type)
                .map_err(|error| SignerError::signing_error(format!("PSBT sighash: {error}")))?;
            let signature = ecdsa::Signature {
                signature: secp.sign_ecdsa_low_r(&Message::from_digest(*sighash.as_ref()), &secret_key),
                sighash_type: input.sighash_type,
            };
            psbt.inputs[input.index].partial_sigs.insert(bitcoin::PublicKey::new(public_key), signature);
        }
        Ok(psbt)
    }

    fn wallet_inputs(&self, psbt: &Psbt, address: &str, script: &ScriptBuf) -> Result<Vec<WalletInput>, SignerError> {
        if self.account != address {
            return SignerError::invalid_input_err("signPsbt account is not the session account");
        }
        if self.sign_inputs.is_empty() {
            return SignerError::invalid_input_err("signPsbt names no input to sign");
        }
        let mut indexes = BTreeSet::new();
        self.sign_inputs
            .iter()
            .map(|input| {
                if !indexes.insert(input.index) {
                    return SignerError::invalid_input_err(format!("input {} is listed twice", input.index));
                }
                if input.address != address {
                    return SignerError::invalid_input_err(format!("input {} belongs to {}, not the wallet", input.index, input.address));
                }
                let output = spent_output(psbt, input.index)?;
                if output.script_pubkey != *script {
                    return SignerError::invalid_input_err(format!("input {} does not spend the wallet's coins", input.index));
                }
                Ok(WalletInput {
                    index: input.index,
                    output,
                    sighash_type: sighash_type(input.sighash_types.as_deref())?,
                })
            })
            .collect()
    }
}

struct WalletInput {
    index: usize,
    output: TxOut,
    sighash_type: EcdsaSighashType,
}

fn held_coins(psbt: &Psbt, inputs: &[WalletInput], utxos: &[UTXO]) -> Result<(), SignerError> {
    for input in inputs {
        let outpoint = psbt.unsigned_tx.input.get(input.index).map(|input| input.previous_output);
        let value = BigUint::from(input.output.value.to_sat());
        let held = outpoint.is_some_and(|outpoint| {
            utxos
                .iter()
                .any(|utxo| utxo.transaction_id == outpoint.txid.to_string() && u32::try_from(utxo.vout).ok() == Some(outpoint.vout) && utxo.value == value)
        });
        if !held {
            return SignerError::invalid_input_err(format!("input {} is not a coin the wallet holds at that amount", input.index));
        }
    }
    Ok(())
}

fn recipient(psbt: &Psbt, script: &ScriptBuf) -> Result<Option<String>, SignerError> {
    let recipients: BTreeSet<&ScriptBuf> = psbt.unsigned_tx.output.iter().map(|output| &output.script_pubkey).filter(|output| *output != script && !output.is_op_return()).collect();
    let mut recipients = recipients.into_iter();
    match (recipients.next(), recipients.next()) {
        (None, _) => Ok(None),
        (Some(recipient), None) => Address::from_script(recipient, Network::Bitcoin)
            .map(|address| Some(address.to_string()))
            .map_err(|_| SignerError::invalid_input("PSBT pays a script that is not an address")),
        (Some(_), Some(_)) => SignerError::invalid_input_err("PSBT pays more than one recipient"),
    }
}

fn total(values: impl Iterator<Item = Amount>) -> Result<u64, SignerError> {
    values.map(Amount::to_sat).try_fold(0u64, u64::checked_add).ok_or_else(|| SignerError::invalid_input("PSBT amounts overflow"))
}

pub fn finalize_psbt(mut psbt: Psbt) -> Result<String, SignerError> {
    for (index, input) in psbt.inputs.iter_mut().enumerate() {
        if input.final_script_witness.is_some() || input.final_script_sig.is_some() {
            continue;
        }
        let Some((public_key, signature)) = input.partial_sigs.iter().next().map(|(key, signature)| (key.inner, *signature)) else {
            return SignerError::invalid_input_err(format!("input {index} is not signed, so the transaction can't be sent"));
        };
        input.final_script_witness = Some(Witness::p2wpkh(&signature, &public_key));
        input.partial_sigs.clear();
        input.sighash_type = None;
        input.bip32_derivation.clear();
    }
    let transaction = psbt.extract_tx().map_err(|error| SignerError::signing_error(format!("PSBT extract: {error}")))?;
    Ok(hex::encode(bitcoin::consensus::encode::serialize(&transaction)))
}

fn wallet_script(address: &str) -> Result<ScriptBuf, SignerError> {
    let address = Address::from_str(address)
        .and_then(|address| address.require_network(Network::Bitcoin))
        .map_err(|error| SignerError::invalid_input(format!("invalid Bitcoin address {address}: {error}")))?;
    let script = address.script_pubkey();
    if !script.is_p2wpkh() {
        return SignerError::invalid_input_err("only native SegWit addresses sign PSBTs");
    }
    Ok(script)
}

fn spent_output(psbt: &Psbt, index: usize) -> Result<TxOut, SignerError> {
    let input = psbt.inputs.get(index).ok_or_else(|| SignerError::invalid_input(format!("PSBT has no input {index}")))?;
    if let Some(output) = &input.witness_utxo {
        return Ok(output.clone());
    }
    let previous = psbt.unsigned_tx.input.get(index).map(|input| input.previous_output);
    input
        .non_witness_utxo
        .as_ref()
        .zip(previous)
        .and_then(|(transaction, outpoint)| transaction.output.get(outpoint.vout as usize).cloned())
        .ok_or_else(|| SignerError::invalid_input(format!("PSBT input {index} does not say what it spends")))
}

fn sighash_type(types: Option<&[u32]>) -> Result<EcdsaSighashType, SignerError> {
    match types {
        None | Some([]) => Ok(EcdsaSighashType::All),
        Some([value]) if *value == EcdsaSighashType::All as u32 => Ok(EcdsaSighashType::All),
        Some([value]) if *value == EcdsaSighashType::AllPlusAnyoneCanPay as u32 => Ok(EcdsaSighashType::AllPlusAnyoneCanPay),
        Some(types) => SignerError::invalid_input_err(format!("sighash types {types:?} would let others change the outputs")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::psbt_mock::{mock_psbt, mock_psbt_coins};
    use bitcoin::{Amount, Transaction};
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;

    const RECIPIENT: &str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";

    fn wallet_address() -> String {
        let public_key = PublicKey::from_secret_key(&Secp256k1::new(), &secret_key(&TEST_PRIVATE_KEY).unwrap());
        Address::p2wpkh(&CompressedPublicKey(public_key), Network::Bitcoin).to_string()
    }

    fn request(inputs: &[(u64, &str)], outputs: &[(u64, &str)], sign: &[(usize, Option<Vec<u32>>)]) -> BitcoinPsbtRequest {
        BitcoinPsbtRequest {
            account: wallet_address(),
            psbt: mock_psbt(inputs, outputs),
            sign_inputs: sign
                .iter()
                .map(|(index, sighash_types)| BitcoinPsbtSignInput {
                    address: wallet_address(),
                    index: *index,
                    sighash_types: sighash_types.clone(),
                })
                .collect(),
            broadcast: false,
        }
    }

    #[test]
    fn test_a_payment_from_the_wallet_is_signed_and_can_be_sent() {
        let wallet = wallet_address();
        let request = request(&[(100_000, &wallet)], &[(60_000, RECIPIENT), (39_000, &wallet)], &[(0, None)]);

        assert_eq!(
            request.summary(&wallet).unwrap(),
            PsbtSummary {
                recipient: Some(RECIPIENT.to_string()),
                value: 60_000,
                fee: 1_000
            },
            "the review shows what leaves the wallet and the fee it pays"
        );

        let signed = request.sign(&wallet, &TEST_PRIVATE_KEY, &mock_psbt_coins(&[(100_000, &wallet)])).unwrap();
        let (public_key, signature) = signed.inputs[0].partial_sigs.iter().next().unwrap();
        let sighash = SighashCache::new(&signed.unsigned_tx)
            .p2wpkh_signature_hash(0, &wallet_script(&wallet).unwrap(), Amount::from_sat(100_000), EcdsaSighashType::All)
            .unwrap();
        assert!(Secp256k1::verification_only().verify_ecdsa(&Message::from_digest(*sighash.as_ref()), &signature.signature, &public_key.inner).is_ok());

        let raw = finalize_psbt(signed).unwrap();
        let transaction: Transaction = bitcoin::consensus::encode::deserialize(&hex::decode(&raw).unwrap()).unwrap();
        assert_eq!(transaction.input[0].witness.len(), 2, "a native SegWit spend carries the signature and the key");
    }

    #[test]
    fn test_only_the_wallets_own_inputs_are_signed() {
        let wallet = wallet_address();

        let coins = mock_psbt_coins(&[(100_000, &wallet)]);

        let others = request(&[(100_000, RECIPIENT)], &[(90_000, &wallet)], &[(0, None)]);
        assert!(others.summary(&wallet).is_err(), "an input that spends someone else's coins is refused before the review");
        assert!(others.sign(&wallet, &TEST_PRIVATE_KEY, &coins).is_err());

        let missing = request(&[(100_000, &wallet)], &[(90_000, RECIPIENT)], &[(3, None)]);
        assert!(missing.sign(&wallet, &TEST_PRIVATE_KEY, &coins).is_err());

        let twice = request(&[(100_000, &wallet)], &[(90_000, RECIPIENT)], &[(0, None), (0, None)]);
        assert!(twice.summary(&wallet).is_err(), "an input listed twice is refused");

        let mut other_account = request(&[(100_000, &wallet)], &[(90_000, RECIPIENT)], &[(0, None)]);
        other_account.account = RECIPIENT.to_string();
        assert!(other_account.summary(&wallet).is_err(), "the request must come for the session account");
        assert!(other_account.sign(&wallet, &TEST_PRIVATE_KEY, &coins).is_err());

        let none = request(&[(100_000, &wallet)], &[(90_000, RECIPIENT)], &[]);
        assert!(none.sign(&wallet, &TEST_PRIVATE_KEY, &coins).is_err());
    }

    #[test]
    fn test_only_signatures_that_commit_to_every_output_are_made() {
        let wallet = wallet_address();
        for (types, allowed) in [
            (None, true),
            (Some(vec![1]), true),
            (Some(vec![0x81]), true),
            (Some(vec![2]), false),
            (Some(vec![3]), false),
            (Some(vec![0x82]), false),
            (Some(vec![1, 2]), false),
        ] {
            let request = request(&[(100_000, &wallet)], &[(90_000, RECIPIENT)], &[(0, types.clone())]);
            assert_eq!(request.summary(&wallet).is_ok(), allowed, "{types:?} before the review");
            assert_eq!(request.sign(&wallet, &TEST_PRIVATE_KEY, &mock_psbt_coins(&[(100_000, &wallet)])).is_ok(), allowed, "{types:?}");
        }
    }

    #[test]
    fn test_a_shared_transaction_shows_what_the_wallet_puts_in() {
        let wallet = wallet_address();
        let request = request(&[(100_000, &wallet), (50_000, RECIPIENT)], &[(140_000, RECIPIENT), (9_000, &wallet)], &[(0, None)]);

        assert_eq!(
            request.summary(&wallet).unwrap(),
            PsbtSummary {
                recipient: Some(RECIPIENT.to_string()),
                value: 91_000,
                fee: 0
            },
            "with another party's inputs the fee is not the wallet's alone, so its whole outflow is shown"
        );
        let coins = mock_psbt_coins(&[(100_000, &wallet)]);
        assert!(finalize_psbt(request.sign(&wallet, &TEST_PRIVATE_KEY, &coins).unwrap()).is_err(), "another party's unsigned input can't be sent");
    }

    #[test]
    fn test_an_input_amount_the_wallet_does_not_hold_is_never_signed() {
        let wallet = wallet_address();
        let held = mock_psbt_coins(&[(1_000_000, &wallet), (1_000_000, &wallet)]);
        let understated = request(&[(1_000_000, &wallet), (1_000, &wallet)], &[(1_000_000, RECIPIENT)], &[(0, None), (1, None)]);

        assert_eq!(understated.summary(&wallet).unwrap().fee, 1_000, "the PSBT claims a small fee");
        assert!(understated.check_coins(&wallet, &held).is_err(), "the load refuses an input whose amount is not the coin's");
        assert!(understated.sign(&wallet, &TEST_PRIVATE_KEY, &held).is_err(), "a signature over a wrong amount could be combined into a far larger fee");
    }

    #[test]
    fn test_an_input_labelled_as_the_wallets_counts_only_when_the_wallet_signs_it() {
        let wallet = wallet_address();
        let request = request(&[(1_000_000, &wallet), (980_000, &wallet)], &[(1_000_000, RECIPIENT)], &[(0, None)]);

        assert_eq!(
            request.summary(&wallet).unwrap(),
            PsbtSummary {
                recipient: Some(RECIPIENT.to_string()),
                value: 1_000_000,
                fee: 0
            },
            "an input the wallet does not sign can't turn its outflow into a fee"
        );
        assert!(request.check_coins(&wallet, &mock_psbt_coins(&[(1_000_000, &wallet)])).is_ok());

        let listed = BitcoinPsbtRequest {
            sign_inputs: vec![
                BitcoinPsbtSignInput {
                    address: wallet.clone(),
                    index: 0,
                    sighash_types: None,
                },
                BitcoinPsbtSignInput {
                    address: wallet.clone(),
                    index: 1,
                    sighash_types: None,
                },
            ],
            ..request
        };
        assert!(listed.check_coins(&wallet, &mock_psbt_coins(&[(1_000_000, &wallet)])).is_err(), "a coin the wallet does not hold is refused");
    }

    #[test]
    fn test_a_psbt_the_review_cannot_show_whole_is_refused() {
        let wallet = wallet_address();
        let other = "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq";

        let two_recipients = request(&[(1_000_000, &wallet)], &[(546, RECIPIENT), (990_000, other)], &[(0, None)]);
        assert!(two_recipients.summary(&wallet).is_err(), "only one recipient is scanned, so a second is refused");

        let overflow = request(&[(u64::MAX, &wallet)], &[(u64::MAX, RECIPIENT), (u64::MAX, &wallet)], &[(0, None)]);
        assert!(overflow.summary(&wallet).is_err());
    }
}
