use std::collections::BTreeSet;

use bitcoin::{
    Address, Amount, Network, Psbt, ScriptBuf, TxOut, Witness, ecdsa,
    secp256k1::{Message, PublicKey, Secp256k1},
    sighash::{EcdsaSighashType, SighashCache},
};
use gem_encoding::decode_base64;
use num_bigint::BigUint;
use primitives::{BitcoinChain, SignerError, SignerInput, UTXO, hex};
use serde::Deserialize;

use crate::signer::{
    address::{UnlockingScript, public_key_hash, script_for_address, script_for_public_key_hash},
    transaction::ZeroizedSecretKey,
};

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

    pub fn from_input(input: &SignerInput) -> Result<Self, SignerError> {
        let extra = input.input_type.get_generic_data().map_err(SignerError::invalid_input)?;
        Self::parse(extra.data_as_str().map_err(SignerError::invalid_input)?)
    }

    pub fn decode_psbt(&self) -> Result<Psbt, SignerError> {
        let bytes = decode_base64(&self.psbt).map_err(|error| SignerError::invalid_input(format!("invalid PSBT: {error}")))?;
        Psbt::deserialize(&bytes).map_err(|error| SignerError::invalid_input(format!("invalid PSBT: {error}")))
    }

    pub fn summary(&self, address: &str) -> Result<PsbtSummary, SignerError> {
        let (psbt, script, inputs) = self.read(address)?;
        summarize(&psbt, &script, &inputs)
    }

    pub fn held_summary(&self, address: &str, utxos: &[UTXO]) -> Result<PsbtSummary, SignerError> {
        let (psbt, script, inputs) = self.read(address)?;
        held_coins(&psbt, &inputs, utxos)?;
        summarize(&psbt, &script, &inputs)
    }

    pub fn sign(&self, address: &str, private_key: &[u8], utxos: &[UTXO]) -> Result<Psbt, SignerError> {
        let (mut psbt, script, inputs) = self.read(address)?;
        held_coins(&psbt, &inputs, utxos)?;
        let secp = Secp256k1::signing_only();
        let secret_key = ZeroizedSecretKey::new(BitcoinChain::Bitcoin, private_key)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        if script_for_public_key_hash(UnlockingScript::P2wpkh, public_key_hash(&public_key.serialize())) != script {
            return SignerError::invalid_input_err("the wallet key does not own the session address");
        }
        let mut cache = SighashCache::new(&psbt.unsigned_tx);
        for input in inputs {
            let sighash = cache
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

    fn read(&self, address: &str) -> Result<(Psbt, ScriptBuf, Vec<WalletInput>), SignerError> {
        let psbt = self.decode_psbt()?;
        let script = wallet_script(address)?;
        let inputs = self.wallet_inputs(&psbt, address, &script)?;
        Ok((psbt, script, inputs))
    }

    fn wallet_inputs(&self, psbt: &Psbt, address: &str, script: &ScriptBuf) -> Result<Vec<WalletInput>, SignerError> {
        if self.account != address {
            return SignerError::invalid_input_err("signPsbt account is not the session account");
        }
        if self.sign_inputs.is_empty() {
            return SignerError::invalid_input_err("signPsbt names no input to sign");
        }
        let mut indexes = BTreeSet::new();
        let inputs = self
            .sign_inputs
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
            .collect::<Result<Vec<_>, SignerError>>()?;
        match (0..psbt.inputs.len()).find(|index| !indexes.contains(index) && spent_output(psbt, *index).is_ok_and(|output| output.script_pubkey == *script)) {
            Some(index) => SignerError::invalid_input_err(format!("input {index} spends the wallet's coins but is not signed with the others")),
            None => Ok(inputs),
        }
    }
}

fn summarize(psbt: &Psbt, script: &ScriptBuf, inputs: &[WalletInput]) -> Result<PsbtSummary, SignerError> {
    let spent = total(inputs.iter().map(|input| input.output.value))?;
    let returned = total(psbt.unsigned_tx.output.iter().filter(|output| output.script_pubkey == *script).map(|output| output.value))?;
    let fee = match inputs.len() == psbt.inputs.len() {
        true => spent
            .checked_sub(total(psbt.unsigned_tx.output.iter().map(|output| output.value))?)
            .ok_or_else(|| SignerError::invalid_input("PSBT pays out more than its inputs hold"))?,
        false => 0,
    };
    Ok(PsbtSummary {
        recipient: recipient(psbt, script)?,
        value: spent.saturating_sub(returned).saturating_sub(fee),
        fee,
    })
}

struct WalletInput {
    index: usize,
    output: TxOut,
    sighash_type: EcdsaSighashType,
}

fn held_coins(psbt: &Psbt, inputs: &[WalletInput], utxos: &[UTXO]) -> Result<(), SignerError> {
    for (index, input) in psbt.unsigned_tx.input.iter().enumerate() {
        let outpoint = input.previous_output;
        let coin = utxos.iter().find(|utxo| utxo.transaction_id == outpoint.txid.to_string() && u32::try_from(utxo.vout).ok() == Some(outpoint.vout));
        match (inputs.iter().find(|signed| signed.index == index), coin) {
            (Some(signed), Some(coin)) if coin.value == BigUint::from(signed.output.value.to_sat()) => {}
            (Some(_), _) => return SignerError::invalid_input_err(format!("input {index} is not a coin the wallet holds at that amount")),
            (None, Some(_)) => return SignerError::invalid_input_err(format!("input {index} is a wallet coin the request does not sign")),
            (None, None) => {}
        }
    }
    Ok(())
}

fn recipient(psbt: &Psbt, script: &ScriptBuf) -> Result<Option<String>, SignerError> {
    if psbt.unsigned_tx.output.iter().any(|output| output.script_pubkey.is_op_return()) {
        return SignerError::invalid_input_err("PSBT carries OP_RETURN data the review can't show");
    }
    let recipients: BTreeSet<&ScriptBuf> = psbt.unsigned_tx.output.iter().map(|output| &output.script_pubkey).filter(|output| *output != script).collect();
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
    let script = script_for_address(BitcoinChain::Bitcoin, address)?;
    match script.unlocking_script() {
        Some(UnlockingScript::P2wpkh) => Ok(script.script_pubkey),
        Some(UnlockingScript::P2pkh) | None => SignerError::invalid_input_err("only native SegWit addresses sign PSBTs"),
    }
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
        Some(types) => SignerError::invalid_input_err(format!("sighash types {types:?} would let others change the transaction")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{
        address_mock::mock_p2wpkh_address,
        psbt_mock::{mock_psbt_coins, mock_psbt_request},
    };
    use bitcoin::{Amount, Transaction};
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;

    const RECIPIENT: &str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";

    #[test]
    fn test_a_payment_from_the_wallet_is_signed_and_can_be_sent() {
        let wallet = mock_p2wpkh_address();
        let request = mock_psbt_request(&[(100_000, &wallet)], &[(60_000, RECIPIENT), (39_000, &wallet)], &[(0, None)]);

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
        let wallet = mock_p2wpkh_address();

        let coins = mock_psbt_coins(&[(100_000, &wallet)]);

        let others = mock_psbt_request(&[(100_000, RECIPIENT)], &[(90_000, &wallet)], &[(0, None)]);
        assert!(others.summary(&wallet).is_err(), "an input that spends someone else's coins is refused before the review");
        assert!(others.sign(&wallet, &TEST_PRIVATE_KEY, &coins).is_err());

        let missing = mock_psbt_request(&[(100_000, &wallet)], &[(90_000, RECIPIENT)], &[(3, None)]);
        assert!(missing.sign(&wallet, &TEST_PRIVATE_KEY, &coins).is_err());

        let twice = mock_psbt_request(&[(100_000, &wallet)], &[(90_000, RECIPIENT)], &[(0, None), (0, None)]);
        assert!(twice.summary(&wallet).is_err(), "an input listed twice is refused");

        let mut other_account = mock_psbt_request(&[(100_000, &wallet)], &[(90_000, RECIPIENT)], &[(0, None)]);
        other_account.account = RECIPIENT.to_string();
        assert!(other_account.summary(&wallet).is_err(), "the request must come for the session account");
        assert!(other_account.sign(&wallet, &TEST_PRIVATE_KEY, &coins).is_err());

        let none = mock_psbt_request(&[(100_000, &wallet)], &[(90_000, RECIPIENT)], &[]);
        assert!(none.sign(&wallet, &TEST_PRIVATE_KEY, &coins).is_err());
    }

    #[test]
    fn test_only_signatures_that_commit_to_every_output_are_made() {
        let wallet = mock_p2wpkh_address();
        for (types, allowed) in [
            (None, true),
            (Some(vec![1]), true),
            (Some(vec![0x81]), false),
            (Some(vec![2]), false),
            (Some(vec![3]), false),
            (Some(vec![0x82]), false),
            (Some(vec![1, 2]), false),
        ] {
            let request = mock_psbt_request(&[(100_000, &wallet)], &[(90_000, RECIPIENT)], &[(0, types.clone())]);
            assert_eq!(request.summary(&wallet).is_ok(), allowed, "{types:?} before the review");
            assert_eq!(request.sign(&wallet, &TEST_PRIVATE_KEY, &mock_psbt_coins(&[(100_000, &wallet)])).is_ok(), allowed, "{types:?}");
        }
    }

    #[test]
    fn test_a_shared_transaction_shows_what_the_wallet_puts_in() {
        let wallet = mock_p2wpkh_address();
        let request = mock_psbt_request(&[(100_000, &wallet), (50_000, RECIPIENT)], &[(140_000, RECIPIENT), (9_000, &wallet)], &[(0, None)]);

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
        let wallet = mock_p2wpkh_address();
        let held = mock_psbt_coins(&[(1_000_000, &wallet), (1_000_000, &wallet)]);
        let understated = mock_psbt_request(&[(1_000_000, &wallet), (1_000, &wallet)], &[(1_000_000, RECIPIENT)], &[(0, None), (1, None)]);

        assert_eq!(understated.summary(&wallet).unwrap().fee, 1_000, "the PSBT claims a small fee");
        assert!(understated.held_summary(&wallet, &held).is_err(), "the load refuses an input whose amount is not the coin's");
        assert!(understated.sign(&wallet, &TEST_PRIVATE_KEY, &held).is_err(), "a signature over a wrong amount could be combined into a far larger fee");
    }

    #[test]
    fn test_every_wallet_coin_of_the_transaction_is_signed_in_one_request() {
        let wallet = mock_p2wpkh_address();
        let coins = mock_psbt_coins(&[(1_000_000, &wallet), (1_000_000, &wallet)]);
        let outputs = [(500_000, RECIPIENT), (1_490_000, wallet.as_str())];

        for index in [0, 1] {
            let split = mock_psbt_request(&[(1_000_000, &wallet), (1_000_000, &wallet)], &outputs, &[(index, None)]);
            assert!(split.summary(&wallet).is_err(), "signing one coin at a time would show each review as nothing leaving");
            assert!(split.sign(&wallet, &TEST_PRIVATE_KEY, &coins).is_err());
        }

        let disguised = mock_psbt_request(&[(1_000_000, &wallet), (1_000_000, RECIPIENT)], &outputs, &[(0, None)]);
        assert!(disguised.held_summary(&wallet, &coins).is_err(), "a wallet coin labelled as someone else's is still the wallet's");

        let whole = mock_psbt_request(&[(1_000_000, &wallet), (1_000_000, &wallet)], &outputs, &[(0, None), (1, None)]);
        assert_eq!(
            whole.summary(&wallet).unwrap(),
            PsbtSummary {
                recipient: Some(RECIPIENT.to_string()),
                value: 500_000,
                fee: 10_000
            }
        );
        assert!(whole.sign(&wallet, &TEST_PRIVATE_KEY, &coins).is_ok());
        assert!(whole.held_summary(&wallet, &[]).is_err(), "a coin the wallet does not hold is refused");
    }

    #[test]
    fn test_a_psbt_the_review_cannot_show_whole_is_refused() {
        let wallet = mock_p2wpkh_address();
        let other = "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq";

        let two_recipients = mock_psbt_request(&[(1_000_000, &wallet)], &[(546, RECIPIENT), (990_000, other)], &[(0, None)]);
        assert!(two_recipients.summary(&wallet).is_err(), "only one recipient is scanned, so a second is refused");

        let mut op_return = mock_psbt_request(&[(1_000_000, &wallet)], &[(990_000, RECIPIENT)], &[(0, None)]);
        let mut psbt = op_return.decode_psbt().unwrap();
        psbt.unsigned_tx.output.push(TxOut {
            value: Amount::ZERO,
            script_pubkey: ScriptBuf::new_op_return(b"=:ETH.ETH:0xabc"),
        });
        psbt.outputs.push(Default::default());
        op_return.psbt = gem_encoding::encode_base64(&psbt.serialize());
        assert!(op_return.summary(&wallet).is_err(), "OP_RETURN data the review can't show is refused");

        let overflow = mock_psbt_request(&[(u64::MAX, &wallet)], &[(u64::MAX, RECIPIENT), (u64::MAX, &wallet)], &[(0, None)]);
        assert!(overflow.summary(&wallet).is_err());
    }
}
