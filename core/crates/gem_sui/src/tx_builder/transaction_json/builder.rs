use super::model::{
    FundsWithdrawalInput, TransactionArgument, TransactionCommand, TransactionInput, TransactionObject, WithdrawalReservation, WithdrawalSource, WithdrawalTypeArg,
};
use crate::{SuiError, address::SuiAddress, tx_builder::move_call as sui_move_call};
use gem_encoding::decode_base64;
use std::{collections::HashMap, str::FromStr};
use sui_transaction_builder::{Argument, ObjectInput, TransactionBuilder};
use sui_types::{Digest, TypeTag};

pub(super) enum CommandOutput {
    Single(Argument),
    Nested(Vec<Argument>),
    Empty,
}

pub(super) fn replay_input(txb: &mut TransactionBuilder, index: usize, input: &TransactionInput, object_inputs: &HashMap<usize, ObjectInput>) -> Result<Argument, SuiError> {
    match input {
        TransactionInput::Pure { pure } => {
            Ok(txb.pure_bytes_unique(decode_base64(&pure.bytes).map_err(|err| SuiError::invalid_input(format!("Invalid Sui transaction encoding: {err}")))?))
        }
        TransactionInput::Object { object } => Ok(txb.object(object_input(object)?)),
        TransactionInput::UnresolvedObject { .. } => Ok(txb.object(
            object_inputs
                .get(&index)
                .cloned()
                .ok_or_else(|| SuiError::invalid_input("Missing resolved Sui object input"))?,
        )),
        TransactionInput::FundsWithdrawal { funds_withdrawal } => funds_withdrawal_input(txb, funds_withdrawal),
        TransactionInput::UnresolvedPure { pure } => Err(SuiError::invalid_input(format!("Sui transaction contains unresolved pure input: {pure}"))),
    }
}

fn funds_withdrawal_input(txb: &mut TransactionBuilder, funds_withdrawal: &FundsWithdrawalInput) -> Result<Argument, SuiError> {
    let WithdrawalSource::Sender { .. } = funds_withdrawal.withdraw_from else {
        return Err(SuiError::invalid_input("Unsupported Sui funds withdrawal source"));
    };
    let WithdrawalReservation::MaxAmountU64 { max_amount } = funds_withdrawal.reservation;
    let WithdrawalTypeArg::Balance { balance } = &funds_withdrawal.type_arg;
    let coin_type: TypeTag = balance
        .parse()
        .map_err(|err| SuiError::invalid_input(format!("Invalid Sui withdrawal balance type {balance}: {err}")))?;
    Ok(txb.funds_withdrawal(coin_type, max_amount))
}

pub(super) fn replay_command(txb: &mut TransactionBuilder, command: TransactionCommand, inputs: &[Argument], outputs: &[CommandOutput]) -> Result<CommandOutput, SuiError> {
    match command {
        TransactionCommand::MoveCall { move_call } => {
            let arguments = move_call
                .arguments
                .iter()
                .map(|argument| input_or_output_argument(txb, argument, inputs, outputs))
                .collect::<Result<Vec<_>, _>>()?;
            let type_arguments = move_call.type_arguments.iter().map(String::as_str).collect::<Vec<_>>();
            let output = sui_move_call(
                txb,
                SuiAddress::parse(&move_call.package)?.into(),
                &move_call.module,
                &move_call.function,
                &type_arguments,
                arguments,
            )?;
            Ok(CommandOutput::Single(output))
        }
        TransactionCommand::TransferObjects { transfer_objects } => {
            let objects = transfer_objects
                .objects
                .iter()
                .map(|argument| input_or_output_argument(txb, argument, inputs, outputs))
                .collect::<Result<Vec<_>, _>>()?;
            let address = input_or_output_argument(txb, &transfer_objects.address, inputs, outputs)?;
            txb.transfer_objects(objects, address);
            Ok(CommandOutput::Empty)
        }
        TransactionCommand::SplitCoins { split_coins } => {
            let coin = input_or_output_argument(txb, &split_coins.coin, inputs, outputs)?;
            let amounts = split_coins
                .amounts
                .iter()
                .map(|argument| input_or_output_argument(txb, argument, inputs, outputs))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CommandOutput::Nested(txb.split_coins(coin, amounts)))
        }
        TransactionCommand::MergeCoins { merge_coins } => {
            let destination = input_or_output_argument(txb, &merge_coins.destination, inputs, outputs)?;
            let sources = merge_coins
                .sources
                .iter()
                .map(|argument| input_or_output_argument(txb, argument, inputs, outputs))
                .collect::<Result<Vec<_>, _>>()?;
            txb.merge_coins(destination, sources);
            Ok(CommandOutput::Empty)
        }
        TransactionCommand::MakeMoveVec { make_move_vec } => {
            let type_ = make_move_vec
                .r#type
                .as_deref()
                .map(TypeTag::from_str)
                .transpose()
                .map_err(|err| SuiError::invalid_input(format!("Invalid Sui MakeMoveVec type: {err}")))?;
            let elements = make_move_vec
                .elements
                .iter()
                .map(|argument| input_or_output_argument(txb, argument, inputs, outputs))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CommandOutput::Single(txb.make_move_vec(type_, elements)))
        }
        TransactionCommand::Publish { publish } => Err(SuiError::invalid_input(format!("Unsupported Sui Publish command: {publish}"))),
        TransactionCommand::Upgrade { upgrade } => Err(SuiError::invalid_input(format!("Unsupported Sui Upgrade command: {upgrade}"))),
    }
}

pub(super) fn output_argument(argument: &TransactionArgument, outputs: &[CommandOutput]) -> Result<Argument, SuiError> {
    match argument {
        TransactionArgument::Result { result } => match outputs.get(*result).ok_or_else(|| SuiError::invalid_input("Missing Sui result argument"))? {
            CommandOutput::Single(argument) => Ok(*argument),
            CommandOutput::Nested(_) | CommandOutput::Empty => Err(SuiError::invalid_input("Invalid Sui result argument")),
        },
        TransactionArgument::NestedResult { nested_result } => {
            let output = outputs.get(nested_result[0]).ok_or_else(|| SuiError::invalid_input("Missing Sui nested result argument"))?;
            match output {
                CommandOutput::Single(argument) => argument
                    .to_nested(nested_result[1] + 1)
                    .get(nested_result[1])
                    .copied()
                    .ok_or_else(|| SuiError::invalid_input("Invalid Sui nested result argument")),
                CommandOutput::Nested(arguments) => arguments
                    .get(nested_result[1])
                    .copied()
                    .ok_or_else(|| SuiError::invalid_input("Invalid Sui nested result argument")),
                CommandOutput::Empty => Err(SuiError::invalid_input("Invalid Sui nested result argument")),
            }
        }
        TransactionArgument::GasCoin { .. } | TransactionArgument::Input { .. } => Err(SuiError::invalid_input("Invalid Sui output argument")),
    }
}

fn object_input(object: &TransactionObject) -> Result<ObjectInput, SuiError> {
    match object {
        TransactionObject::ImmOrOwnedObject { object } => Ok(ObjectInput::owned(SuiAddress::parse(&object.object_id)?.into(), object.version, digest(&object.digest)?)),
        TransactionObject::SharedObject { object } => Ok(ObjectInput::shared(
            SuiAddress::parse(&object.object_id)?.into(),
            object.initial_shared_version,
            object.mutable,
        )),
        TransactionObject::Receiving { object } => Ok(ObjectInput::receiving(
            SuiAddress::parse(&object.object_id)?.into(),
            object.version,
            digest(&object.digest)?,
        )),
    }
}

fn input_or_output_argument(txb: &mut TransactionBuilder, argument: &TransactionArgument, inputs: &[Argument], outputs: &[CommandOutput]) -> Result<Argument, SuiError> {
    match argument {
        TransactionArgument::GasCoin { gas_coin } => {
            if *gas_coin {
                Ok(txb.gas())
            } else {
                Err(SuiError::invalid_input("Invalid Sui gas coin argument"))
            }
        }
        TransactionArgument::Input { input } => inputs.get(*input).copied().ok_or_else(|| SuiError::invalid_input("Missing Sui input argument")),
        TransactionArgument::Result { .. } | TransactionArgument::NestedResult { .. } => output_argument(argument, outputs),
    }
}

pub(super) fn digest(value: &str) -> Result<Digest, SuiError> {
    Digest::from_str(value).map_err(|err| SuiError::invalid_input(format!("Invalid Sui object digest {value}: {err}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_funds_withdrawal_input() {
        let input: TransactionInput = serde_json::from_str(include_str!("../../../testdata/funds_withdrawal_sender.json")).unwrap();

        let mut txb = TransactionBuilder::new();
        assert!(replay_input(&mut txb, 0, &input, &HashMap::new()).is_ok());

        let sponsored: TransactionInput = serde_json::from_str(include_str!("../../../testdata/funds_withdrawal_sponsor.json")).unwrap();

        let mut txb = TransactionBuilder::new();
        assert!(replay_input(&mut txb, 0, &sponsored, &HashMap::new()).is_err());
    }
}
