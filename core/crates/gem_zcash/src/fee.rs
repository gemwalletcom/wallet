use gem_bitcoin::signer::{PlanInput, PlanOutput, SpendFee};

const MARGINAL_FEE: u128 = 5_000;
const GRACE_ACTIONS: u128 = 2;
const P2PKH_INPUT_SIZE: u128 = 150;
const P2PKH_OUTPUT_SIZE: u128 = 34;
const SHIELDED_MIN_ACTIONS: u128 = 2;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Zip317Fee;

impl SpendFee for Zip317Fee {
    fn estimate_fee(&self, inputs: &[PlanInput], outputs: &[PlanOutput], external_outputs: usize) -> u64 {
        let input_total_size = inputs.len() as u128 * P2PKH_INPUT_SIZE;
        let output_total_size: u128 = outputs.iter().map(|output| output.serialized_len() as u128).sum();
        let input_actions = input_total_size.div_ceil(P2PKH_INPUT_SIZE);
        let output_actions = output_total_size.div_ceil(P2PKH_OUTPUT_SIZE);
        let logical_actions = input_actions.max(output_actions) + Self::shielded_actions(external_outputs);
        (MARGINAL_FEE * GRACE_ACTIONS.max(logical_actions)) as u64
    }
}

impl Zip317Fee {
    fn shielded_actions(shielded_outputs: usize) -> u128 {
        match shielded_outputs {
            0 => 0,
            outputs => SHIELDED_MIN_ACTIONS.max(outputs as u128),
        }
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::{
        signer::address::{UnlockingScript, script_for_public_key_hash},
        testkit::planner_mock::op_return_script,
    };

    use super::*;

    #[test]
    fn test_estimate_fee() {
        let input = vec![PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh)];
        let inputs = vec![
            PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh),
            PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh),
            PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh),
        ];
        let outputs = vec![PlanOutput::new(10_000, script_for_public_key_hash(UnlockingScript::P2pkh, [0u8; 20]))];
        let large_output = vec![PlanOutput::new(0, op_return_script(80))];

        assert_eq!(Zip317Fee.estimate_fee(&input, &outputs, 0), 10_000);
        assert_eq!(Zip317Fee.estimate_fee(&inputs, &outputs, 0), 15_000);
        assert_eq!(Zip317Fee.estimate_fee(&input, &large_output, 0), 15_000);
        assert_eq!(Zip317Fee.estimate_fee(&input, &[], 1), 15_000);
        assert_eq!(Zip317Fee.estimate_fee(&input, &outputs, 1), 15_000);
        assert_eq!(Zip317Fee.estimate_fee(&inputs, &outputs, 1), 25_000);
    }
}
