use gem_bitcoin::signer::{PlanInput, PlanOutput, SpendFee};

const MARGINAL_FEE: u128 = 5_000;
const GRACE_ACTIONS: u128 = 2;
const P2PKH_OUTPUT_SIZE: u128 = 34;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Zip317Fee;

impl SpendFee for Zip317Fee {
    fn estimate_fee(&self, inputs: &[PlanInput], outputs: &[PlanOutput]) -> u64 {
        let output_size: u128 = outputs.iter().map(|output| output.serialized_len() as u128).sum();
        let logical_actions = (inputs.len() as u128).max(output_size.div_ceil(P2PKH_OUTPUT_SIZE));
        (MARGINAL_FEE * GRACE_ACTIONS.max(logical_actions)) as u64
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::{signer::address::UnlockingScript, testkit::planner_mock::op_return_script};

    use super::*;

    #[test]
    fn test_estimate_fee() {
        let input = PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh);
        let output = PlanOutput::mock_with_unlocking_script(UnlockingScript::P2pkh, 10_000);

        assert_eq!(Zip317Fee.estimate_fee(std::slice::from_ref(&input), std::slice::from_ref(&output)), 10_000);
        assert_eq!(Zip317Fee.estimate_fee(&[input.clone(), input.clone(), input.clone()], std::slice::from_ref(&output)), 15_000);
        assert_eq!(Zip317Fee.estimate_fee(&[input], &[PlanOutput::new(0, op_return_script(80))]), 15_000);
    }
}
