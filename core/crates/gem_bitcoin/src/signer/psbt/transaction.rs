use std::collections::HashSet;

use bitcoin::{Address, Network, Psbt, sighash::EcdsaSighashType};
use num_traits::ToPrimitive;
use primitives::{BitcoinChain, Chain, SignerError, SignerInput};

use crate::signer::address::script_for_address;

pub fn parse_transaction(psbt_hex: &str, sender: &str, value: u64) -> Result<(Psbt, String, u64), SignerError> {
    let bytes = hex::decode(psbt_hex).map_err(SignerError::from_display)?;
    let psbt = Psbt::deserialize(&bytes).map_err(SignerError::from_display)?;
    let sender_script = script_for_address(BitcoinChain::Bitcoin, sender)?.script_pubkey;
    if value == 0 || psbt.inputs.is_empty() || !sender_script.is_p2wpkh() {
        return SignerError::invalid_input_err("unsupported Bitcoin PSBT sender or amount");
    }
    let mut outpoints = HashSet::new();
    for (transaction_input, input) in psbt.unsigned_tx.input.iter().zip(&psbt.inputs) {
        let previous_output = input.witness_utxo.as_ref().ok_or_else(|| SignerError::invalid_input("missing Bitcoin PSBT witness UTXO"))?;
        if previous_output.script_pubkey != sender_script || !outpoints.insert(transaction_input.previous_output) {
            return SignerError::invalid_input_err("Bitcoin PSBT inputs do not match sender");
        }
        if input.ecdsa_hash_ty() != Ok(EcdsaSighashType::All) {
            return SignerError::invalid_input_err("unsupported Bitcoin PSBT sighash");
        }
    }
    let mut destination = None;
    for output in &psbt.unsigned_tx.output {
        if output.script_pubkey == sender_script {
            continue;
        }
        if output.script_pubkey.is_op_return() {
            if output.value.to_sat() != 0 {
                return SignerError::invalid_input_err("Bitcoin PSBT OP_RETURN output must have zero value");
            }
            continue;
        }
        if destination.is_some() || output.value.to_sat() != value {
            return SignerError::invalid_input_err("Bitcoin PSBT outputs do not match swap amount");
        }
        destination = Some(
            Address::from_script(&output.script_pubkey, Network::Bitcoin)
                .map_err(SignerError::from_display)?
                .to_string(),
        );
    }
    let destination = destination.ok_or_else(|| SignerError::invalid_input("missing Bitcoin PSBT deposit output"))?;
    let fee = psbt.fee().map_err(SignerError::from_display)?.to_sat();
    Ok((psbt, destination, fee))
}

pub fn validate_swap_transaction(chain: BitcoinChain, input: &SignerInput) -> Result<(Psbt, u64), SignerError> {
    let swap = input.input_type.get_swap_data()?;
    if chain.get_chain() != Chain::Bitcoin {
        return SignerError::invalid_input_err("unsupported Bitcoin contract swap");
    }
    let asset = input.input_type.get_asset();
    if asset.chain() != chain.get_chain() || !asset.id.is_native() {
        return SignerError::invalid_input_err("unsupported Bitcoin swap asset");
    }
    let is_max = swap.quote.use_max_amount.unwrap_or(input.is_max_value);
    if swap.quote.from_address != input.sender_address
        || swap.data.value != swap.quote.from_value
        || if is_max { swap.data.value > input.value } else { swap.data.value != input.value }
    {
        return SignerError::invalid_input_err("Bitcoin PSBT does not match swap quote");
    }
    let value = swap.data.value.to_u64().ok_or_else(|| SignerError::invalid_input("invalid Bitcoin swap amount"))?;
    let (psbt, destination, fee) = parse_transaction(&swap.data.data, &input.sender_address, value)?;
    if script_for_address(chain, &destination)?.script_pubkey != script_for_address(chain, &swap.data.to)?.script_pubkey {
        return SignerError::invalid_input_err("Bitcoin PSBT does not match swap destination");
    }
    let utxos = input.metadata.get_utxos()?;
    for (transaction_input, previous_output) in psbt.unsigned_tx.input.iter().zip(psbt.iter_funding_utxos()) {
        let previous_output = previous_output.map_err(SignerError::from_display)?;
        let outpoint = transaction_input.previous_output;
        let transaction_id = outpoint.txid.to_string();
        let utxo = utxos
            .iter()
            .find(|utxo| utxo.transaction_id == transaction_id && u32::try_from(utxo.vout) == Ok(outpoint.vout))
            .ok_or_else(|| SignerError::invalid_input("unknown Bitcoin PSBT input"))?;
        if utxo.value_u64().map_err(SignerError::from_display)? != previous_output.value.to_sat()
            || script_for_address(chain, &utxo.address)?.script_pubkey != previous_output.script_pubkey
        {
            return SignerError::invalid_input_err("Bitcoin PSBT input does not match UTXO");
        }
    }
    Ok((psbt, fee))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_deposit_fixture() {
        let psbt = include_str!("../../../testdata/relay_deposit.hex").trim();
        let (_, address, fee) = parse_transaction(psbt, "bc1qq2mvrp4g3ugd424dw4xv53rgsf8szkrv853jrc", 2_000_000).unwrap();
        assert_eq!((address, fee), ("bc1qzmtn0q92ayejt2hpffvlktcpmyy7vvsd06sefu".to_string(), 3108));
    }

    #[test]
    fn test_reject_invalid_psbt() {
        let sender = "bc1qq2mvrp4g3ugd424dw4xv53rgsf8szkrv853jrc";
        assert!(parse_transaction("not hex", sender, 10_000).is_err());
        assert!(parse_transaction("deadbeef", sender, 10_000).is_err());
    }
}
