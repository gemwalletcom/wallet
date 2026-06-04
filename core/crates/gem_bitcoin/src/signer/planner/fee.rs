use primitives::BitcoinChain;

use super::{PlanInput, PlanOutput};
use crate::signer::{address::UnlockingScript, encoding::varint_len};

const WITNESS_SCALE_FACTOR: u128 = 4;
const TRANSACTION_FIXED_BYTES: u128 = 8;
const SEGWIT_MARKER_FLAG_WEIGHT: u128 = 2;
const P2PKH_INPUT_BYTES: u128 = 148;
const P2WPKH_INPUT_BASE_BYTES: u128 = 41;
const P2WPKH_INPUT_WITNESS_BYTES: u128 = 108;

const ZCASH_MARGINAL_FEE: u128 = 5_000;
const ZCASH_GRACE_ACTIONS: u128 = 2;
const ZCASH_P2PKH_INPUT_SIZE: u128 = 150;
const ZCASH_P2PKH_OUTPUT_SIZE: u128 = 34;

pub(super) fn estimate_fee(chain: BitcoinChain, inputs: &[PlanInput], outputs: &[PlanOutput], fee_rate: u64) -> u64 {
    let fee = match chain {
        BitcoinChain::Zcash => estimate_zcash_fee(inputs, outputs),
        BitcoinChain::Bitcoin | BitcoinChain::BitcoinCash | BitcoinChain::Litecoin | BitcoinChain::Doge => estimate_bitcoin_fee(inputs, outputs, fee_rate),
    };
    fee as u64
}

fn estimate_bitcoin_fee(inputs: &[PlanInput], outputs: &[PlanOutput], fee_rate: u64) -> u128 {
    let has_witness = inputs.iter().any(|input| match input.unlocking_script {
        UnlockingScript::P2wpkh => true,
        UnlockingScript::P2pkh => false,
    });
    let input_weight: u128 = inputs.iter().map(input_weight).sum();
    let output_weight: u128 = outputs.iter().map(|output| output.serialized_len() as u128 * WITNESS_SCALE_FACTOR).sum();
    let weight = transaction_base_weight(inputs.len(), outputs.len(), has_witness) + input_weight + output_weight;
    weight.div_ceil(WITNESS_SCALE_FACTOR) * fee_rate as u128
}

fn transaction_base_weight(input_count: usize, output_count: usize, has_witness: bool) -> u128 {
    let base_bytes = TRANSACTION_FIXED_BYTES + varint_len(input_count) as u128 + varint_len(output_count) as u128;
    let witness_weight = if has_witness { SEGWIT_MARKER_FLAG_WEIGHT } else { 0 };
    base_bytes * WITNESS_SCALE_FACTOR + witness_weight
}

fn input_weight(input: &PlanInput) -> u128 {
    match input.unlocking_script {
        UnlockingScript::P2pkh => P2PKH_INPUT_BYTES * WITNESS_SCALE_FACTOR,
        UnlockingScript::P2wpkh => P2WPKH_INPUT_BASE_BYTES * WITNESS_SCALE_FACTOR + P2WPKH_INPUT_WITNESS_BYTES,
    }
}

fn estimate_zcash_fee(inputs: &[PlanInput], outputs: &[PlanOutput]) -> u128 {
    let input_total_size = inputs.len() as u128 * ZCASH_P2PKH_INPUT_SIZE;
    let output_total_size: u128 = outputs.iter().map(|output| output.serialized_len() as u128).sum();
    let input_actions = input_total_size.div_ceil(ZCASH_P2PKH_INPUT_SIZE);
    let output_actions = output_total_size.div_ceil(ZCASH_P2PKH_OUTPUT_SIZE);
    let logical_actions = input_actions.max(output_actions);
    ZCASH_MARGINAL_FEE * ZCASH_GRACE_ACTIONS.max(logical_actions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{signer::address::script_for_public_key_hash, testkit::planner_mock::op_return_script};

    #[test]
    fn test_estimate_fee() {
        let legacy_inputs = vec![PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh)];
        let legacy_outputs = vec![
            PlanOutput::new(10_000, script_for_public_key_hash(UnlockingScript::P2pkh, [0u8; 20])),
            PlanOutput::new(20_000, script_for_public_key_hash(UnlockingScript::P2pkh, [0u8; 20])),
        ];
        assert_eq!(estimate_fee(BitcoinChain::Bitcoin, &legacy_inputs, &legacy_outputs, 2), 452);

        let segwit_inputs = vec![PlanInput::mock_with_unlocking_script(UnlockingScript::P2wpkh)];
        let segwit_outputs = vec![PlanOutput::new(10_000, script_for_public_key_hash(UnlockingScript::P2wpkh, [0u8; 20]))];
        assert_eq!(estimate_fee(BitcoinChain::Bitcoin, &segwit_inputs, &segwit_outputs, 3), 330);

        let zcash_outputs = vec![PlanOutput::new(10_000, script_for_public_key_hash(UnlockingScript::P2pkh, [0u8; 20]))];
        assert_eq!(estimate_fee(BitcoinChain::Zcash, &legacy_inputs, &zcash_outputs, 100), 10_000);

        let zcash_inputs = vec![
            PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh),
            PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh),
            PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh),
        ];
        assert_eq!(estimate_fee(BitcoinChain::Zcash, &zcash_inputs, &zcash_outputs, 100), 15_000);

        let zcash_large_output = vec![PlanOutput::new(0, op_return_script(80))];
        assert_eq!(estimate_fee(BitcoinChain::Zcash, &legacy_inputs, &zcash_large_output, 100), 15_000);
    }
}
