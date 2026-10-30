use bitcoin::ScriptBuf;
use primitives::{BitcoinChain, SignerError, SignerInput};

use super::{
    PlanInput, PlanOutput, SpendDestination, SpendFee, SpendPlan, SpendRequest, VirtualSizeFee,
    inputs::spendable_inputs,
    outputs::{Payment, dust_threshold},
};
use crate::signer::address::script_for_address;

pub struct UtxoPlanner;

#[derive(Debug, Clone, Copy)]
enum SpendTarget {
    Exact(u64),
    Max,
}

impl UtxoPlanner {
    pub(crate) fn transfer(chain: BitcoinChain, input: &SignerInput) -> Result<SpendPlan, SignerError> {
        let request = SpendRequest::transfer(chain, input, |address| SpendDestination::script(chain, address))?;
        Self::plan(request, &VirtualSizeFee::for_input(chain, input)?)
    }

    pub(crate) fn swap(chain: BitcoinChain, input: &SignerInput) -> Result<SpendPlan, SignerError> {
        let request = SpendRequest::swap(chain, input, |address| SpendDestination::script(chain, address))?;
        Self::plan(request, &VirtualSizeFee::for_input(chain, input)?)
    }

    #[cfg(any(test, feature = "rpc"))]
    pub fn fee(request: SpendRequest, fee: &impl SpendFee) -> Result<u64, SignerError> {
        match Self::plan(request.clone(), fee) {
            Err(SignerError::InsufficientFunds) => Self::fee_spending_all_inputs(request, fee),
            plan => plan.map(|plan| plan.fee),
        }
    }

    #[cfg(any(test, feature = "rpc"))]
    fn fee_spending_all_inputs(request: SpendRequest, fee: &impl SpendFee) -> Result<u64, SignerError> {
        let payment = Payment::new(request.destination, request.memo.as_deref())?;
        let (mut outputs, _) = payment.outputs(request.amount);
        if !request.is_max {
            outputs.push(PlanOutput::new(0, script_for_address(request.chain, &request.sender_address)?.script_pubkey));
        }
        let inputs = spendable_inputs(request.chain, &request.sender_address, request.utxos)?;
        Ok(fee.estimate_fee(&inputs, &outputs, payment.external_outputs()))
    }

    pub fn plan(request: SpendRequest, fee: &impl SpendFee) -> Result<SpendPlan, SignerError> {
        if request.utxos.is_empty() {
            return SignerError::invalid_input_err("missing input UTXOs");
        }
        if !request.is_max && request.amount == 0 {
            return SignerError::invalid_input_err("invalid transaction amount");
        }

        let payment = Payment::new(request.destination, request.memo.as_deref())?;
        if !request.is_max && request.amount < payment.dust_threshold(request.chain) {
            return Err(SignerError::DustThreshold);
        }

        let change_script = script_for_address(request.chain, &request.sender_address)?.script_pubkey;
        let spendable_inputs = spendable_inputs(request.chain, &request.sender_address, request.utxos)?;
        let target = if request.is_max { SpendTarget::Max } else { SpendTarget::Exact(request.amount) };
        Self::select_inputs_and_build_plan(request.chain, target, fee, &payment, change_script, spendable_inputs)
    }

    fn select_inputs_and_build_plan(chain: BitcoinChain, target: SpendTarget, fee: &impl SpendFee, payment: &Payment, change_script: ScriptBuf, mut spendable_inputs: Vec<PlanInput>) -> Result<SpendPlan, SignerError> {
        // Smallest-first so Exact selects the fewest inputs; Max spends all, so sorting just keeps it deterministic.
        spendable_inputs.sort_by(|left, right| left.value.to_sat().cmp(&right.value.to_sat()).then_with(|| left.previous_output.cmp(&right.previous_output)));

        let input_count = spendable_inputs.len();
        let total: u128 = spendable_inputs.iter().map(|input| input.value.to_sat() as u128).sum();
        if total > u64::MAX as u128 {
            return SignerError::invalid_input_err("Bitcoin amount overflow");
        }
        let mut selected = Vec::new();
        let mut selected_amount = 0u64;
        for (index, candidate) in spendable_inputs.into_iter().enumerate() {
            selected_amount += candidate.value.to_sat();
            selected.push(candidate);

            let value = match target {
                // Max spends every input, so only build the plan once all inputs are gathered.
                SpendTarget::Max if index + 1 < input_count => continue,
                SpendTarget::Max => return Self::build_max_plan(chain, fee, payment, &selected, selected_amount),
                SpendTarget::Exact(value) => value,
            };
            if let Some(plan) = Self::build_exact_plan(chain, value, fee, payment, &change_script, &selected, selected_amount)? {
                return Ok(plan);
            }
        }

        Err(SignerError::InsufficientFunds)
    }

    fn build_max_plan(chain: BitcoinChain, fee: &impl SpendFee, payment: &Payment, selected: &[PlanInput], selected_amount: u64) -> Result<SpendPlan, SignerError> {
        // Output value doesn't affect fee size; size the fee from the output shape, then spend the rest.
        let (outputs, _) = payment.outputs(0);
        let fee = fee.estimate_fee(selected, &outputs, payment.external_outputs());
        let value = selected_amount.checked_sub(fee).filter(|value| *value > 0).ok_or(SignerError::InsufficientFunds)?;
        if value < payment.dust_threshold(chain) {
            return Err(SignerError::DustThreshold);
        }
        let (outputs, external_value) = payment.outputs(value);
        Ok(SpendPlan {
            inputs: selected.to_vec(),
            outputs,
            external_value,
            fee,
        })
    }

    fn build_exact_plan(chain: BitcoinChain, value: u64, fee: &impl SpendFee, payment: &Payment, change_script: &ScriptBuf, selected: &[PlanInput], selected_amount: u64) -> Result<Option<SpendPlan>, SignerError> {
        let (mut outputs, external_value) = payment.outputs(value);
        let external_outputs = payment.external_outputs();

        let mut outputs_with_change = outputs.clone();
        outputs_with_change.push(PlanOutput::new(0, change_script.clone()));

        let fee_with_change = fee.estimate_fee(selected, &outputs_with_change, external_outputs);
        let Some(remainder) = selected_amount.checked_sub(value).and_then(|remaining| remaining.checked_sub(fee_with_change)) else {
            return Ok(None);
        };

        if remainder >= dust_threshold(chain, change_script) {
            outputs.push(PlanOutput::new(remainder, change_script.clone()));
            return Ok(Some(SpendPlan {
                inputs: selected.to_vec(),
                outputs,
                external_value,
                fee: fee_with_change,
            }));
        }
        let fee_without_change = fee.estimate_fee(selected, &outputs, external_outputs);
        let Some(remainder) = selected_amount.checked_sub(value).and_then(|remaining| remaining.checked_sub(fee_without_change)) else {
            return Ok(None);
        };
        Ok(Some(SpendPlan {
            inputs: selected.to_vec(),
            outputs,
            external_value,
            fee: fee_without_change + remainder,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{
        address_mock::{TEST_BITCOIN_P2WPKH_ADDRESS, mock_destination_address, mock_sender_address},
        planner_mock::{mock_signer_input, mock_signer_input_with, mock_spend_utxos, sum_inputs},
        signer_mock::{TEST_UTXO_TXID, mock_transfer_input_with_utxos, mock_utxo_with},
    };
    use num_bigint::BigInt;
    use primitives::{AssetId, Chain, GasPriceType, TransactionFee};

    fn plan_with_fee_rate(value: &str, fee_rate: u64) -> SpendPlan {
        let mut input = mock_signer_input(value, false);
        input.input.gas_price = GasPriceType::regular(BigInt::from(fee_rate));
        input.fee = TransactionFee::new_from_fee(BigInt::from(fee_rate), AssetId::from_chain(Chain::Bitcoin));
        UtxoPlanner::transfer(BitcoinChain::Bitcoin, &input).unwrap()
    }

    fn transfer_fee(input: &SignerInput) -> Result<u64, SignerError> {
        let chain = BitcoinChain::Bitcoin;
        let request = SpendRequest::transfer(chain, input, |address| SpendDestination::script(chain, address))?;
        UtxoPlanner::fee(request, &VirtualSizeFee::for_input(chain, input)?)
    }

    #[test]
    fn test_plan_applies_fractional_fee_rate_below_one_sat() {
        let sub_one = plan_with_fee_rate("5000", 1);
        let one_sat = plan_with_fee_rate("5000", 10);

        assert!(sub_one.fee > 0);
        assert!(sub_one.fee < one_sat.fee);
        assert_eq!(sub_one.outputs[0].value.to_sat(), 5_000);
    }

    #[test]
    fn test_plan_transfer() {
        let plan = UtxoPlanner::transfer(BitcoinChain::Bitcoin, &mock_signer_input("12000", false)).unwrap();
        assert_eq!(plan.inputs.len(), 2);
        assert_eq!(plan.outputs.len(), 3);
        assert_eq!(plan.outputs[0].value.to_sat(), 12_000);
        assert_eq!(plan.outputs[1].value.to_sat(), 0);
        assert!(plan.outputs[1].script_pubkey.is_op_return());
        assert_eq!(sum_inputs(&plan.inputs).unwrap(), plan.outputs.iter().map(|output| output.value.to_sat()).sum::<u64>() + plan.fee);
        assert_eq!(plan.fee, 454);

        // Leftover is below P2WPKH change dust (~294), so it is absorbed into the fee.
        let plan = UtxoPlanner::transfer(BitcoinChain::Bitcoin, &mock_signer_input("9600", false)).unwrap();
        assert_eq!(plan.inputs.len(), 1);
        assert_eq!(plan.outputs.len(), 2);
        assert_eq!(plan.outputs[0].value.to_sat(), 9_600);
        assert!(plan.outputs[1].script_pubkey.is_op_return());
        assert_eq!(sum_inputs(&plan.inputs).unwrap(), 9_600 + plan.outputs[1].value.to_sat() + plan.fee);
        assert_eq!(plan.outputs[1].value.to_sat(), 0);

        assert_eq!(UtxoPlanner::transfer(BitcoinChain::Bitcoin, &mock_signer_input("50000", false)).unwrap_err(), SignerError::InsufficientFunds);
        assert_eq!(
            transfer_fee(&mock_signer_input("50000", false)).unwrap(),
            454,
            "the fee load prices a send the coins can't cover, so the amount check can say Insufficient Balance"
        );

        assert_eq!(transfer_fee(&mock_signer_input("545", false)).unwrap_err(), SignerError::DustThreshold);

        let dust_max_utxos = vec![mock_utxo_with(TEST_UTXO_TXID, 0, "600", TEST_BITCOIN_P2WPKH_ADDRESS)];
        assert_eq!(
            transfer_fee(&mock_signer_input_with("0", true, None, dust_max_utxos)).unwrap_err(),
            SignerError::DustThreshold,
            "a Max that leaves only dust to send is dust"
        );

        assert_eq!(
            UtxoPlanner::transfer(BitcoinChain::Bitcoin, &mock_signer_input_with("1000", false, Some("a".repeat(81)), mock_spend_utxos())).unwrap_err(),
            SignerError::invalid_input("Bitcoin memo is too large")
        );

        let plan = UtxoPlanner::transfer(BitcoinChain::Bitcoin, &mock_signer_input("0", true)).unwrap();
        assert_eq!(plan.inputs.len(), 2);
        assert_eq!(plan.outputs.len(), 2);
        assert!(plan.outputs[1].script_pubkey.is_op_return());
        assert_eq!(sum_inputs(&plan.inputs).unwrap(), plan.outputs.iter().map(|output| output.value.to_sat()).sum::<u64>() + plan.fee);
    }

    #[test]
    fn test_plan_absorbs_sub_dust_change_into_fee() {
        let input = mock_signer_input("9600", false);
        let change_script = script_for_address(BitcoinChain::Bitcoin, &input.sender_address).unwrap().script_pubkey;
        let plan = UtxoPlanner::transfer(BitcoinChain::Bitcoin, &input).unwrap();
        let fee = VirtualSizeFee::for_input(BitcoinChain::Bitcoin, &input).unwrap();

        assert_eq!(plan.outputs.len(), 2);
        assert_eq!(plan.outputs[0].value.to_sat(), 9_600);
        assert!(plan.outputs[1].script_pubkey.is_op_return());

        let selected_amount = sum_inputs(&plan.inputs).unwrap();
        let fee_without_change = fee.estimate_fee(&plan.inputs, &plan.outputs, 0);
        let mut outputs_with_change = plan.outputs.clone();
        outputs_with_change.push(PlanOutput::new(0, change_script.clone()));
        let fee_with_change = fee.estimate_fee(&plan.inputs, &outputs_with_change, 0);
        let dust_remainder = selected_amount - plan.outputs[0].value.to_sat() - fee_with_change;
        assert!(dust_remainder > 0);
        assert!(dust_remainder < dust_threshold(BitcoinChain::Bitcoin, &change_script));

        let absorbed_remainder = selected_amount - plan.outputs[0].value.to_sat() - fee_without_change;
        assert_eq!(plan.fee, fee_without_change + absorbed_remainder);
        assert_eq!(selected_amount, plan.outputs.iter().map(|output| output.value.to_sat()).sum::<u64>() + plan.fee);
    }

    #[test]
    fn test_doge_plan_absorbs_change_below_doge_dust() {
        let sender_address = mock_sender_address(BitcoinChain::Doge);
        let destination_address = mock_destination_address(BitcoinChain::Doge);
        let input = mock_transfer_input_with_utxos(BitcoinChain::Doge, &sender_address, &destination_address, "1500000", vec![mock_utxo_with(TEST_UTXO_TXID, 0, "2000000", &sender_address)]);
        let plan = UtxoPlanner::transfer(BitcoinChain::Doge, &input).unwrap();

        assert_eq!(plan.inputs.len(), 1);
        assert_eq!(plan.outputs.len(), 1);
        assert_eq!(plan.outputs[0].value.to_sat(), 1_500_000);
        assert_eq!(sum_inputs(&plan.inputs).unwrap(), plan.outputs.iter().map(|output| output.value.to_sat()).sum::<u64>() + plan.fee);
    }

    #[test]
    fn test_dust_threshold_is_script_aware() {
        let p2wpkh = script_for_address(BitcoinChain::Bitcoin, TEST_BITCOIN_P2WPKH_ADDRESS).unwrap().script_pubkey;
        let p2pkh = script_for_address(BitcoinChain::Bitcoin, "1BoatSLRHtKNngkdXEeobR76b53LETtpyT").unwrap().script_pubkey;
        let doge_p2pkh = script_for_address(BitcoinChain::Doge, &mock_destination_address(BitcoinChain::Doge)).unwrap().script_pubkey;

        assert_eq!(dust_threshold(BitcoinChain::Bitcoin, &p2pkh), 546);
        assert_eq!(dust_threshold(BitcoinChain::Doge, &doge_p2pkh), 546_000);
        assert!(dust_threshold(BitcoinChain::Bitcoin, &p2wpkh) > 0 && dust_threshold(BitcoinChain::Bitcoin, &p2wpkh) < dust_threshold(BitcoinChain::Bitcoin, &p2pkh));
    }
}
