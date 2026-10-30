use primitives::{BitcoinChain, FeeUnitType, SignerError, SignerInput};

use super::{PlanInput, PlanOutput};
use crate::signer::{address::UnlockingScript, encoding::varint_len};

const WITNESS_SCALE_FACTOR: u128 = 4;
const TRANSACTION_FIXED_BYTES: u128 = 8;
const SEGWIT_MARKER_FLAG_WEIGHT: u128 = 2;
const P2PKH_INPUT_BYTES: u128 = 148;
const P2WPKH_INPUT_BASE_BYTES: u128 = 41;
const P2WPKH_INPUT_WITNESS_BYTES: u128 = 108;

pub trait SpendFee {
    fn estimate_fee(&self, inputs: &[PlanInput], outputs: &[PlanOutput], external_outputs: usize) -> u64;
}

#[derive(Debug, Clone, Copy)]
pub struct VirtualSizeFee {
    fee_rate: u64,
}

impl VirtualSizeFee {
    pub fn new(fee_rate: u64) -> Self {
        Self { fee_rate }
    }

    pub fn for_input(chain: BitcoinChain, input: &SignerInput) -> Result<Self, SignerError> {
        Ok(Self::new(input.fee.gas_price_u64()?.max(u64::from(chain.minimum_custom_fee_rate()))))
    }
}

impl SpendFee for VirtualSizeFee {
    fn estimate_fee(&self, inputs: &[PlanInput], outputs: &[PlanOutput], _external_outputs: usize) -> u64 {
        let has_witness = inputs.iter().any(|input| match input.unlocking_script {
            UnlockingScript::P2wpkh => true,
            UnlockingScript::P2pkh => false,
        });
        let input_weight: u128 = inputs.iter().map(input_weight).sum();
        let output_weight: u128 = outputs.iter().map(|output| output.serialized_len() as u128 * WITNESS_SCALE_FACTOR).sum();
        let weight = transaction_base_weight(inputs.len(), outputs.len(), has_witness) + input_weight + output_weight;
        let scale = FeeUnitType::SatVb.scale_factor() as u128;
        (weight.div_ceil(WITNESS_SCALE_FACTOR) * self.fee_rate as u128).div_ceil(scale) as u64
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::address::script_for_public_key_hash;

    #[test]
    fn test_estimate_fee() {
        let legacy_inputs = vec![PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh)];
        let legacy_outputs = vec![
            PlanOutput::new(10_000, script_for_public_key_hash(UnlockingScript::P2pkh, [0u8; 20])),
            PlanOutput::new(20_000, script_for_public_key_hash(UnlockingScript::P2pkh, [0u8; 20])),
        ];
        assert_eq!(VirtualSizeFee::new(20).estimate_fee(&legacy_inputs, &legacy_outputs, 0), 452);
        assert_eq!(VirtualSizeFee::new(1).estimate_fee(&legacy_inputs, &legacy_outputs, 0), 23);

        let segwit_inputs = vec![PlanInput::mock_with_unlocking_script(UnlockingScript::P2wpkh)];
        let segwit_outputs = vec![PlanOutput::new(10_000, script_for_public_key_hash(UnlockingScript::P2wpkh, [0u8; 20]))];
        assert_eq!(VirtualSizeFee::new(30).estimate_fee(&segwit_inputs, &segwit_outputs, 0), 330);
        assert_eq!(VirtualSizeFee::new(5).estimate_fee(&segwit_inputs, &segwit_outputs, 0), 55);
        assert_eq!(VirtualSizeFee::new(1).estimate_fee(&segwit_inputs, &segwit_outputs, 0), 11);
    }
}
