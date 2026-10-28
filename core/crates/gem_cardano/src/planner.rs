use num_traits::ToPrimitive;
use primitives::{Chain, SignerError, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, UTXO, hex::decode_hex_array};

use crate::{
    address::ShelleyAddress,
    transaction::{Transaction, TransactionInput, TransactionOutput},
};

const CARDANO_EXPIRATION_BLOCK_OFFSET: u64 = 7_200;
pub(crate) const FEE_ESTIMATE_LOVELACE: u64 = 170_000;
pub(crate) const MIN_OUTPUT_LOVELACE: u64 = 1_000_000;
const FEE_CONSTANT_MILLI_LOVELACE: u64 = 155_881_000;
const FEE_COEFFICIENT_MILLI_LOVELACE: u64 = 44_046;
const MILLI_LOVELACE: u64 = 1_000;

pub(crate) struct TransactionPlan {
    utxos: Vec<UTXO>,
    amount: u64,
    pub(crate) fee: u64,
    change: u64,
}

pub(crate) fn transfer_fee(input: &TransactionLoadInput) -> Result<u64, SignerError> {
    Ok(select_transfer(input)?.1)
}

pub(crate) fn plan_transfer(input: &TransactionLoadInput) -> Result<TransactionPlan, SignerError> {
    let (utxos, fee) = select_transfer(input)?;
    let available_amount = sum_amounts(&utxos)?;
    let amount = if input.is_max_value {
        available_amount.checked_sub(fee).ok_or(SignerError::InsufficientFunds)?
    } else {
        input.value_as_u64()?
    };
    let change = available_amount
        .checked_sub(amount)
        .and_then(|remaining| remaining.checked_sub(fee))
        .filter(|change| *change == 0 || *change >= MIN_OUTPUT_LOVELACE)
        .ok_or(SignerError::InsufficientFunds)?;

    Ok(TransactionPlan { utxos, amount, fee, change })
}

fn select_transfer(input: &TransactionLoadInput) -> Result<(Vec<UTXO>, u64), SignerError> {
    match &input.input_type {
        TransactionInputType::Transfer { asset } | TransactionInputType::Withdrawal { asset } if asset.id.chain == Chain::Cardano && asset.id.is_native() => {}
        TransactionInputType::Swap { from_asset: asset, .. } if asset.id.chain == Chain::Cardano && asset.id.is_native() => {}
        TransactionInputType::Transfer { .. } | TransactionInputType::Withdrawal { .. } => return SignerError::invalid_input_err("unsupported Cardano token transfer"),
        TransactionInputType::Swap { .. } => return SignerError::invalid_input_err("unsupported Cardano token swap"),
        _ => return SignerError::invalid_input_err("unsupported Cardano transaction type"),
    }

    let requested_amount = input.value_as_u64()?;

    let utxos = utxos_from_metadata(&input.metadata, &input.sender_address)?;
    if utxos.is_empty() {
        return SignerError::invalid_input_err("missing input UTXOs");
    }

    let selected_utxos = select_utxos(&utxos, requested_amount, input.is_max_value)?;
    let fee = estimate_fee(input, &selected_utxos, requested_amount, input.is_max_value)?;
    Ok((selected_utxos, fee))
}

pub(crate) fn transaction_from_plan(input: &TransactionLoadInput, plan: &TransactionPlan) -> Result<Transaction, SignerError> {
    let destination = ShelleyAddress::parse(transaction_destination_address(input))?;
    let expiration_block_number = input.metadata.get_block_number()? + CARDANO_EXPIRATION_BLOCK_OFFSET;
    let mut outputs = vec![TransactionOutput {
        address: destination.as_bytes().to_vec(),
        amount: plan.amount,
    }];

    if plan.change > 0 {
        let change = ShelleyAddress::parse(&input.sender_address)?;
        outputs.push(TransactionOutput {
            address: change.as_bytes().to_vec(),
            amount: plan.change,
        });
    }

    Ok(Transaction {
        inputs: plan.utxos.iter().map(utxo_transaction_input).collect::<Result<Vec<_>, _>>()?,
        outputs,
        fee: plan.fee,
        expiration_block_number,
        memo: transaction_memo(input),
    })
}

fn transaction_destination_address(input: &TransactionLoadInput) -> &str {
    match &input.input_type {
        TransactionInputType::Swap { swap_data, .. } => &swap_data.data.to,
        _ => &input.destination_address,
    }
}

fn transaction_memo(input: &TransactionLoadInput) -> Option<String> {
    input.get_memo().map(ToOwned::to_owned).or_else(|| input.input_type.get_swap_data().ok().and_then(|swap_data| swap_data.data.memo.clone()))
}

fn utxos_from_metadata(metadata: &TransactionLoadMetadata, sender_address: &str) -> Result<Vec<UTXO>, SignerError> {
    let sender = ShelleyAddress::parse(sender_address)?;
    let utxos = metadata.get_utxos()?;
    for utxo in &utxos {
        utxo_transaction_input(utxo)?;
        utxo_amount(utxo)?;
        let address = ShelleyAddress::parse(&utxo.address)?;
        if address.payment_hash() != sender.payment_hash() {
            return SignerError::invalid_input_err("Cardano UTXO address does not match sender address");
        }
    }
    Ok(utxos)
}

fn utxo_transaction_input(utxo: &UTXO) -> Result<TransactionInput, SignerError> {
    let transaction_hash = decode_hex_array(&utxo.transaction_id).map_err(|_| SignerError::invalid_input("invalid Cardano UTXO transaction id"))?;
    let output_index = utxo.vout.to_u64().ok_or_else(|| SignerError::invalid_input("invalid Cardano UTXO output index"))?;
    Ok(TransactionInput { transaction_hash, output_index })
}

fn utxo_amount(utxo: &UTXO) -> Result<u64, SignerError> {
    let amount = utxo.value_u64().map_err(SignerError::from_display)?;
    if amount == 0 {
        return SignerError::invalid_input_err("invalid Cardano UTXO amount");
    }
    Ok(amount)
}

fn select_utxos(utxos: &[UTXO], amount: u64, max_amount: bool) -> Result<Vec<UTXO>, SignerError> {
    if max_amount {
        return Ok(utxos.to_vec());
    }

    let target = amount
        .checked_mul(4)
        .and_then(|value| value.checked_div(3))
        .and_then(|value| value.checked_add(FEE_ESTIMATE_LOVELACE))
        .and_then(|value| value.checked_add(MIN_OUTPUT_LOVELACE))
        .ok_or_else(|| SignerError::invalid_input("Cardano amount overflow"))?;
    let mut candidates = utxos.iter().map(|utxo| Ok((utxo.clone(), utxo_amount(utxo)?))).collect::<Result<Vec<_>, SignerError>>()?;
    candidates.sort_by(|(_, left_amount), (_, right_amount)| right_amount.cmp(left_amount));

    let mut selected = Vec::new();
    let mut selected_amount = 0u64;
    for (utxo, amount) in candidates {
        selected_amount = selected_amount.checked_add(amount).ok_or_else(|| SignerError::invalid_input("Cardano amount overflow"))?;
        selected.push(utxo);
        if selected_amount >= target {
            break;
        }
    }
    if selected.is_empty() {
        return SignerError::invalid_input_err("missing input UTXOs");
    }
    Ok(selected)
}

fn estimate_fee(input: &TransactionLoadInput, selected_utxos: &[UTXO], requested_amount: u64, max_amount: bool) -> Result<u64, SignerError> {
    let available_amount = sum_amounts(selected_utxos)?;
    let amount = if max_amount {
        available_amount.saturating_sub(FEE_ESTIMATE_LOVELACE)
    } else {
        requested_amount.min(available_amount.saturating_sub(FEE_ESTIMATE_LOVELACE))
    };
    let change = available_amount.saturating_sub(amount).saturating_sub(FEE_ESTIMATE_LOVELACE);
    let plan = TransactionPlan {
        utxos: selected_utxos.to_vec(),
        amount,
        fee: FEE_ESTIMATE_LOVELACE,
        change,
    };
    let transaction = transaction_from_plan(input, &plan)?;
    Ok(transaction_fee(transaction.signed_size() as u64))
}

fn transaction_fee(transaction_size: u64) -> u64 {
    (FEE_CONSTANT_MILLI_LOVELACE + transaction_size * FEE_COEFFICIENT_MILLI_LOVELACE).div_ceil(MILLI_LOVELACE)
}

fn sum_amounts(utxos: &[UTXO]) -> Result<u64, SignerError> {
    utxos
        .iter()
        .try_fold(0u64, |sum, utxo| sum.checked_add(utxo_amount(utxo)?).ok_or_else(|| SignerError::invalid_input("Cardano amount overflow")))
}

#[cfg(test)]
mod tests {
    use num_bigint::{BigInt, BigUint};
    use primitives::{Asset, AssetType, Chain, SwapProvider, TransactionInputType, TransferAmount, TransferAmountError, TransferAmountInput, swap::SwapData};

    use super::*;
    use crate::testkit::{TEST_BLOCK_NUMBER, mock_cardano_metadata};

    const OWN_ADDRESS_1: &str = "addr1q8043m5heeaydnvtmmkyuhe6qv5havvhsf0d26q3jygsspxlyfpyk6yqkw0yhtyvtr0flekj84u64az82cufmqn65zdsylzk23";
    const TO_ADDRESS: &str = "addr1q92cmkgzv9h4e5q7mnrzsuxtgayvg4qr7y3gyx97ukmz3dfx7r9fu73vqn25377ke6r0xk97zw07dqr9y5myxlgadl2s0dgke5";
    const TEST_EXPIRATION_BLOCK_NUMBER: u64 = TEST_BLOCK_NUMBER + CARDANO_EXPIRATION_BLOCK_OFFSET;

    fn shown_amount(input: &TransactionLoadInput, available: u64) -> Result<TransferAmount, TransferAmountError> {
        TransferAmountInput {
            fee: BigInt::from(transfer_fee(input).unwrap()),
            is_max_amount: input.is_max_value,
            output_minimum: input.metadata.get_output_minimum(),
            ..TransferAmountInput::mock(input.input_type.clone(), input.value_as_u64().unwrap(), available, available)
        }
        .calculate()
    }

    #[test]
    fn test_near_empty_send_is_signed_as_shown() {
        let near_empty = |value: u64, balance: u64| TransactionLoadInput {
            metadata: mock_cardano_metadata(vec![UTXO {
                transaction_id: "f074134aabbfb13b8aec7cf5465b1e5a862bde5cb88532cc7e64619179b3e767".to_string(),
                vout: 1,
                value: BigUint::from(balance),
                address: OWN_ADDRESS_1.to_string(),
            }]),
            ..TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, &value.to_string())
        };
        for (value, balance) in [(9_330_000u64, 10_000_000u64), (9_800_000, 10_000_000)] {
            let input = near_empty(value, balance);
            assert_eq!(plan_transfer(&input).err(), Some(SignerError::InsufficientFunds), "{value}: the requested send cannot keep a valid change");
            assert!(shown_amount(&input, balance).unwrap().is_max_amount, "{value}: what is left is under 1 ADA, so the send becomes a send of everything");

            let send_everything = TransactionLoadInput {
                value: BigUint::from(balance),
                is_max_value: true,
                ..input
            };
            let shown = shown_amount(&send_everything, balance).unwrap();
            let signed = plan_transfer(&send_everything).unwrap();
            assert_eq!((shown.value, shown.network_fee), (BigInt::from(signed.amount), BigInt::from(signed.fee)));
            assert_eq!((signed.amount + signed.fee, signed.change), (balance, 0));
            assert!(signed.amount >= value, "{value}: never less than the amount entered");
        }

        assert!(
            matches!(shown_amount(&near_empty(12_200_000, 12_220_000), 12_220_000), Err(TransferAmountError::InsufficientBalance { .. })),
            "12.2 of 12.22 ADA: the fee doesn't fit next to the amount, so it is refused rather than sending less"
        );

        assert_eq!(
            shown_amount(&TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "999999"), 8_000_000),
            Err(TransferAmountError::DustThreshold {
                asset_id: Asset::from_chain(Chain::Cardano).id,
                minimum: BigInt::from(MIN_OUTPUT_LOVELACE),
                value: BigInt::from(999_999),
            })
        );
        assert_eq!(
            shown_amount(&TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "2000000"), 8_000_000).unwrap(),
            TransferAmount {
                value: BigInt::from(2_000_000),
                network_fee: BigInt::from(168_655),
                is_max_amount: false,
            }
        );
    }

    #[test]
    fn test_plan_transfer_vectors() {
        let input = TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "7000000");
        assert_eq!(transfer_fee(&input), Ok(170_240));
        assert_eq!(plan_transfer(&input).err(), Some(SignerError::InsufficientFunds), "0.83 ADA of change is under the minimum output");

        let plan = plan_transfer(&TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "1")).unwrap();
        assert_eq!(plan.utxos.len(), 1);
        assert_eq!(sum_amounts(&plan.utxos).unwrap(), 6_500_000);
        assert_eq!(plan.amount, 1);
        assert_eq!(plan.fee, 168_479);
        assert_eq!(plan.change, 6_331_520);

        let plan = plan_transfer(&TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "2000000")).unwrap();
        assert_eq!(plan.utxos.len(), 1);
        assert_eq!(sum_amounts(&plan.utxos).unwrap(), 6_500_000);
        assert_eq!(plan.amount, 2_000_000);
        assert_eq!(plan.fee, 168_655);
        assert_eq!(plan.change, 4_331_345);

        let plan = plan_transfer(&TransactionLoadInput {
            is_max_value: true,
            ..TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "2000000")
        })
        .unwrap();
        assert_eq!(plan.utxos.len(), 2);
        assert_eq!(sum_amounts(&plan.utxos).unwrap(), 8_000_000);
        assert_eq!(plan.amount, 7_832_622);
        assert_eq!(plan.fee, 167_378);
        assert_eq!(plan.change, 0);
        assert_eq!(plan.utxos[0].value, BigUint::from(1500000u64));
        assert_eq!(plan.utxos[1].value, BigUint::from(6500000u64));
    }

    #[test]
    fn test_plan_transfer_android_vector_fee() {
        let address = "addr1q9d2dxen8ywvs9yzxxn2w4mvffn797fquauvugt2ug7mfsuqj3lzdq9h0rsketzszrnfm930658swmpe7kpq53c2tmwql4rvtq";
        let input = TransactionLoadInput {
            destination_address: address.to_string(),
            metadata: mock_cardano_metadata(vec![UTXO {
                address: address.to_string(),
                transaction_id: "412c5a964cf4515210bf4b82f45df6521c38e1e5381f27638fc509bef6679378".to_string(),
                value: BigUint::from(7945975u64),
                vout: 1,
            }]),
            ..TransactionLoadInput::mock_cardano(address, "10000")
        };

        let plan = plan_transfer(&input).unwrap();
        assert_eq!(plan.fee, 168_567);
        assert_eq!(plan.change, 7_767_408);
    }

    #[test]
    fn test_plan_transfer_accepts_native_swap() {
        let mut input = TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "2000000");
        let mut swap_data = SwapData::mock_transfer(SwapProvider::Mayachain, "2000000", "1000000000000000", TO_ADDRESS);
        swap_data.data.memo = Some("=:e:0x1234567890abcdef:0/1/0:g1:50".to_string());
        input.input_type = TransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::Cardano),
            to_asset: Asset::from_chain(Chain::Ethereum),
            swap_data,
        };
        input.destination_address = "0x1234567890abcdef".to_string();

        let plan = plan_transfer(&input).unwrap();
        let transaction = transaction_from_plan(&input, &plan).unwrap();

        assert_eq!(plan.amount, 2_000_000);
        assert_eq!(plan.fee, 172_531);
        assert_eq!(transaction.outputs[0].address, ShelleyAddress::parse(TO_ADDRESS).unwrap().as_bytes());
        assert_eq!(transaction.memo, Some("=:e:0x1234567890abcdef:0/1/0:g1:50".to_string()));
    }

    #[test]
    fn test_plan_transfer_validation() {
        let mut input = TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "1");
        input.metadata = mock_cardano_metadata(vec![]);
        assert!(plan_transfer(&input).is_err());

        input = TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "1");
        input.metadata = mock_cardano_metadata(vec![UTXO {
            transaction_id: "zz".to_string(),
            vout: 0,
            value: BigUint::from(1u64),
            address: OWN_ADDRESS_1.to_string(),
        }]);
        assert!(plan_transfer(&input).is_err());

        input = TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "1");
        input.destination_address = "stake1uykptcz226y5r5at5rfqqm00p9n0z0yfajz3gk3j3wm8dxg2sn0r4".to_string();
        assert!(plan_transfer(&input).is_err());

        input = TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "1");
        input.input_type = TransactionInputType::Transfer {
            asset: Asset::mock_with_params(Chain::Cardano, Some("policy.asset".to_string()), "Cardano Token".to_string(), "TOKEN".to_string(), 0, AssetType::TOKEN),
        };
        assert_eq!(plan_transfer(&input).err().unwrap().to_string(), "Invalid input: unsupported Cardano token transfer");

        input = TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "1");
        input.input_type = TransactionInputType::Swap {
            from_asset: Asset::mock_with_params(Chain::Cardano, Some("policy.asset".to_string()), "Cardano Token".to_string(), "TOKEN".to_string(), 0, AssetType::TOKEN),
            to_asset: Asset::from_chain(Chain::Ethereum),
            swap_data: SwapData::mock_transfer(SwapProvider::Mayachain, "1", "1", TO_ADDRESS),
        };
        assert_eq!(plan_transfer(&input).err().unwrap().to_string(), "Invalid input: unsupported Cardano token swap");

        input = TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "1");
        input.metadata = mock_cardano_metadata(vec![UTXO {
            transaction_id: "f074134aabbfb13b8aec7cf5465b1e5a862bde5cb88532cc7e64619179b3e767".to_string(),
            vout: 1,
            value: BigUint::from(1500000u64),
            address: TO_ADDRESS.to_string(),
        }]);
        assert_eq!(plan_transfer(&input).err().unwrap().to_string(), "Invalid input: Cardano UTXO address does not match sender address");
    }

    #[test]
    fn test_transaction_expiration_block_number() {
        let input = TransactionLoadInput::mock_cardano(OWN_ADDRESS_1, "1");
        let plan = plan_transfer(&input).unwrap();
        let transaction = transaction_from_plan(&input, &plan).unwrap();
        assert_eq!(transaction.expiration_block_number, TEST_EXPIRATION_BLOCK_NUMBER);
    }
}
