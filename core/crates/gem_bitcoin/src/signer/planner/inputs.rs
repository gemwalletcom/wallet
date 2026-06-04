use std::str::FromStr;

use bitcoin::{Amount, OutPoint, Txid};
use primitives::{BitcoinChain, SignerError, UTXO};

use super::PlanInput;
use crate::signer::address::script_for_address;

const FINAL_SEQUENCE: u32 = 0xffff_ffff;
const RBF_SEQUENCE: u32 = 0xffff_fffd;

pub(super) fn spendable_inputs(chain: BitcoinChain, sender_address: &str, utxos: Vec<UTXO>) -> Result<Vec<PlanInput>, SignerError> {
    let message = |reason: &str| format!("{} {reason}", chain.get_chain());

    let sender = script_for_address(chain, sender_address)?;
    let sender_hash = sender
        .unlocking_script()
        .zip(sender.public_key_hash())
        .map(|(_, public_key_hash)| public_key_hash)
        .ok_or_else(|| SignerError::invalid_input(message("sender address type is unsupported")))?;
    // RBF is signaled on every BTC-family chain; Zcash has no RBF and keeps the final sequence.
    let sequence = match chain {
        BitcoinChain::Bitcoin | BitcoinChain::BitcoinCash | BitcoinChain::Litecoin | BitcoinChain::Doge => RBF_SEQUENCE,
        BitcoinChain::Zcash => FINAL_SEQUENCE,
    };
    let mut inputs = Vec::with_capacity(utxos.len());

    for utxo in utxos {
        let address = script_for_address(chain, &utxo.address)?;
        let (unlocking_script, public_key_hash) = address
            .unlocking_script()
            .zip(address.public_key_hash())
            .ok_or_else(|| SignerError::invalid_input(message("UTXO address type is unsupported")))?;
        if public_key_hash != sender_hash {
            return SignerError::invalid_input_err(message("UTXO address does not match sender address"));
        }
        let value = utxo.value_u64().map_err(SignerError::from_display)?;
        if value == 0 {
            return SignerError::invalid_input_err(message("UTXO amount is zero"));
        }
        let vout = u32::try_from(utxo.vout).map_err(|_| SignerError::invalid_input(message("UTXO output index is invalid")))?;
        let txid = Txid::from_str(&utxo.transaction_id).map_err(|_| SignerError::invalid_input(message("UTXO transaction id is invalid")))?;
        inputs.push(PlanInput {
            previous_output: OutPoint::new(txid, vout),
            value: Amount::from_sat(value),
            script_pubkey: address.script_pubkey,
            unlocking_script,
            sequence,
        });
    }

    Ok(inputs)
}

#[cfg(test)]
mod tests {
    use crate::{
        signer::address::UnlockingScript,
        testkit::{
            address_mock::{TEST_BITCOIN_P2WPKH_ADDRESS, TEST_BITCOIN_P2WPKH_HASH, prefixed_address},
            signer_mock::mock_utxo_with_address,
        },
    };

    use super::*;

    const TAPROOT_ADDRESS: &str = "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr";

    #[test]
    fn test_spendable_inputs_address_type_validation() {
        let legacy_address = prefixed_address(&[0], TEST_BITCOIN_P2WPKH_HASH);
        let inputs = spendable_inputs(
            BitcoinChain::Bitcoin,
            TEST_BITCOIN_P2WPKH_ADDRESS,
            vec![mock_utxo_with_address(&legacy_address), mock_utxo_with_address(TEST_BITCOIN_P2WPKH_ADDRESS)],
        )
        .unwrap();
        assert_eq!(inputs[0].unlocking_script, UnlockingScript::P2pkh);
        assert_eq!(inputs[1].unlocking_script, UnlockingScript::P2wpkh);

        let different_legacy_address = prefixed_address(&[0], [9u8; 20]);
        assert_eq!(
            spendable_inputs(BitcoinChain::Bitcoin, TEST_BITCOIN_P2WPKH_ADDRESS, vec![mock_utxo_with_address(&different_legacy_address)],).err(),
            Some(SignerError::invalid_input("bitcoin UTXO address does not match sender address")),
        );
        assert_eq!(
            spendable_inputs(BitcoinChain::Bitcoin, TEST_BITCOIN_P2WPKH_ADDRESS, vec![mock_utxo_with_address(TAPROOT_ADDRESS)]).err(),
            Some(SignerError::invalid_input("bitcoin UTXO address type is unsupported")),
        );
        assert_eq!(
            spendable_inputs(BitcoinChain::Bitcoin, TAPROOT_ADDRESS, vec![mock_utxo_with_address(TEST_BITCOIN_P2WPKH_ADDRESS)]).err(),
            Some(SignerError::invalid_input("bitcoin sender address type is unsupported")),
        );
    }
}
