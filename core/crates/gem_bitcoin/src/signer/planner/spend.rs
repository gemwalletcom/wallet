use bitcoin::{Amount, ScriptBuf};
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

struct Spend<'a, F> {
    chain: BitcoinChain,
    target: SpendTarget,
    fee: &'a F,
    payment: Payment,
    change_script: ScriptBuf,
    inputs: Vec<PlanInput>,
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

    pub fn plan(request: SpendRequest, fee: &impl SpendFee) -> Result<SpendPlan, SignerError> {
        Spend::new(request, fee)?.plan().ok_or(SignerError::InsufficientFunds)
    }

    pub fn fee(request: SpendRequest, fee: &impl SpendFee) -> Result<u64, SignerError> {
        let spend = Spend::new(request, fee)?;
        Ok(match spend.plan() {
            Some(plan) => plan.fee,
            None => spend.estimate_fee(&spend.inputs, &spend.outputs().0),
        })
    }
}

impl<'a, F: SpendFee> Spend<'a, F> {
    fn new(request: SpendRequest, fee: &'a F) -> Result<Self, SignerError> {
        if request.utxos.is_empty() {
            return SignerError::invalid_input_err("missing input UTXOs");
        }

        let payment = Payment::new(request.destination, request.memo.as_deref())?;
        let change_script = script_for_address(request.chain, &request.sender_address)?.script_pubkey;
        let mut inputs = spendable_inputs(request.chain, &request.sender_address, request.utxos)?;
        inputs.sort_by(|left, right| left.value.to_sat().cmp(&right.value.to_sat()).then_with(|| left.previous_output.cmp(&right.previous_output)));
        if inputs.iter().map(|input| input.value.to_sat() as u128).sum::<u128>() > u64::MAX as u128 {
            return SignerError::invalid_input_err("Bitcoin amount overflow");
        }

        Ok(Self {
            chain: request.chain,
            target: if request.is_max { SpendTarget::Max } else { SpendTarget::Exact(request.amount) },
            fee,
            payment,
            change_script,
            inputs,
        })
    }

    fn plan(&self) -> Option<SpendPlan> {
        match self.target {
            SpendTarget::Max => self.plan_with(&self.inputs),
            SpendTarget::Exact(_) => (1..=self.inputs.len()).find_map(|count| self.plan_with(&self.inputs[..count])),
        }
    }

    fn plan_with(&self, inputs: &[PlanInput]) -> Option<SpendPlan> {
        let (mut outputs, mut external_value) = self.outputs();
        let fee = self.estimate_fee(inputs, &outputs);
        let spendable = inputs.iter().map(|input| input.value.to_sat()).sum::<u64>().checked_sub(fee)?;
        match self.target {
            SpendTarget::Max => (outputs, external_value) = self.payment.outputs(spendable),
            SpendTarget::Exact(value) => {
                let change = spendable.checked_sub(value).filter(|change| *change >= dust_threshold(self.chain, &self.change_script))?;
                outputs.last_mut()?.value = Amount::from_sat(change);
            }
        }
        Some(SpendPlan {
            inputs: inputs.to_vec(),
            outputs,
            external_value,
            fee,
        })
    }

    fn outputs(&self) -> (Vec<PlanOutput>, Option<u64>) {
        match self.target {
            SpendTarget::Max => self.payment.outputs(0),
            SpendTarget::Exact(value) => {
                let (mut outputs, external_value) = self.payment.outputs(value);
                outputs.push(PlanOutput::new(0, self.change_script.clone()));
                (outputs, external_value)
            }
        }
    }

    fn estimate_fee(&self, inputs: &[PlanInput], outputs: &[PlanOutput]) -> u64 {
        self.fee.estimate_fee(inputs, outputs, self.payment.external_outputs())
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
    use primitives::{AssetId, Chain, GasPriceType, TransactionFee, TransferAmount, TransferAmountError, TransferAmountInput};

    fn plan_with_fee_rate(value: &str, fee_rate: u64) -> SpendPlan {
        let mut input = mock_signer_input(value, false);
        input.input.gas_price = GasPriceType::regular(BigInt::from(fee_rate));
        input.fee = TransactionFee::new_from_fee(BigInt::from(fee_rate), AssetId::from_chain(Chain::Bitcoin));
        UtxoPlanner::transfer(BitcoinChain::Bitcoin, &input).unwrap()
    }

    fn transfer_fee(chain: BitcoinChain, input: &SignerInput) -> u64 {
        let request = SpendRequest::transfer(chain, input, |address| SpendDestination::script(chain, address)).unwrap();
        UtxoPlanner::fee(request, &VirtualSizeFee::for_input(chain, input).unwrap()).unwrap()
    }

    fn shown_amount(chain: BitcoinChain, input: &SignerInput, available: u64) -> Result<TransferAmount, TransferAmountError> {
        TransferAmountInput {
            fee: BigInt::from(transfer_fee(chain, input)),
            is_max_amount: input.is_max_value,
            output_minimum: input.metadata.get_output_minimum(),
            ..TransferAmountInput::mock(input.input_type.clone(), input.value_as_u64().unwrap(), available, available)
        }
        .calculate()
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
        assert_eq!(
            shown_amount(BitcoinChain::Bitcoin, &mock_signer_input("12000", false), 30_000).unwrap(),
            TransferAmount {
                value: BigInt::from(12_000),
                network_fee: BigInt::from(454),
                is_max_amount: false,
            }
        );

        let plan = UtxoPlanner::transfer(BitcoinChain::Bitcoin, &mock_signer_input("9600", false)).unwrap();
        assert_eq!(plan.inputs.len(), 2, "the first input alone would leave dust change, so another input keeps a valid change");
        assert_eq!(plan.outputs[0].value.to_sat(), 9_600);
        assert_eq!(plan.outputs[2].value.to_sat(), 30_000 - 9_600 - plan.fee);

        assert_eq!(UtxoPlanner::transfer(BitcoinChain::Bitcoin, &mock_signer_input("50000", false)).unwrap_err(), SignerError::InsufficientFunds);
        assert_eq!(
            transfer_fee(BitcoinChain::Bitcoin, &mock_signer_input("50000", false)),
            454,
            "the fee load prices the send it was asked for instead of refusing it"
        );

        let dust_max_utxos = vec![mock_utxo_with(TEST_UTXO_TXID, 0, "600", TEST_BITCOIN_P2WPKH_ADDRESS)];
        let dust_max = mock_signer_input_with("600", true, None, dust_max_utxos);
        assert_eq!(
            shown_amount(BitcoinChain::Bitcoin, &dust_max, 600),
            Err(TransferAmountError::DustThreshold {
                asset_id: AssetId::from_chain(Chain::Bitcoin),
                minimum: BigInt::from(546),
                value: BigInt::from(374),
            })
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
    fn test_amount_under_dust() {
        assert_eq!(
            shown_amount(BitcoinChain::Bitcoin, &mock_signer_input("545", false), 30_000),
            Err(TransferAmountError::DustThreshold {
                asset_id: AssetId::from_chain(Chain::Bitcoin),
                minimum: BigInt::from(546),
                value: BigInt::from(545),
            })
        );
        assert!(shown_amount(BitcoinChain::Bitcoin, &mock_signer_input("546", false), 30_000).is_ok());
    }

    #[test]
    fn test_near_empty_send_is_signed_as_shown() {
        let doge_sender = mock_sender_address(BitcoinChain::Doge);
        let doge = mock_transfer_input_with_utxos(
            BitcoinChain::Doge,
            &doge_sender,
            &mock_destination_address(BitcoinChain::Doge),
            "1500000",
            vec![mock_utxo_with(TEST_UTXO_TXID, 0, "2000000", &doge_sender)],
        );
        let cases = [
            (BitcoinChain::Bitcoin, mock_signer_input("29446", false), 30_000),
            (BitcoinChain::Bitcoin, mock_signer_input("29300", false), 30_000),
            (BitcoinChain::Bitcoin, mock_signer_input("29900", false), 30_000),
            (BitcoinChain::Doge, doge, 2_000_000),
        ];
        for (chain, input, balance) in cases {
            let value = input.value.clone();
            assert_eq!(UtxoPlanner::transfer(chain, &input).unwrap_err(), SignerError::InsufficientFunds, "{chain:?} {value}");
            assert!(
                shown_amount(chain, &input, balance).unwrap().is_max_amount,
                "{chain:?} {value}: what is left is under the change dust, so the send becomes a send of everything"
            );

            let mut send_everything = input;
            send_everything.input.value = balance.into();
            send_everything.input.is_max_value = true;
            let shown = shown_amount(chain, &send_everything, balance).unwrap();
            let signed = UtxoPlanner::transfer(chain, &send_everything).unwrap();
            assert_eq!((shown.value, shown.network_fee), (BigInt::from(signed.outputs[0].value.to_sat()), BigInt::from(signed.fee)), "{chain:?} {value}");
            assert_eq!(signed.outputs.iter().map(|output| output.value.to_sat()).sum::<u64>() + signed.fee, balance, "{chain:?} {value}: no change is left");
        }
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
