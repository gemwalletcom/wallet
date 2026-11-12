use std::collections::HashMap;

use serde::Serialize;

use crate::{
    SuiError,
    types::{
        Address, Argument, Command, FundsWithdrawal, Input, MakeMoveVector, MergeCoins, MoveCall, Mutability, ObjectInput, ProgrammableTransaction, SplitCoins, TransferObjects, TypeTag, WithdrawFrom, WithdrawalReservation, WithdrawalType,
    },
};

const MAX_ARGUMENTS: usize = u16::MAX as usize + 1;

#[derive(Default)]
pub struct TransactionBuilder {
    inputs: Vec<Input>,
    commands: Vec<Command>,
    pure_inputs: HashMap<Vec<u8>, Argument>,
    object_inputs: HashMap<Address, usize>,
    error: Option<String>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gas(&self) -> Argument {
        Argument::Gas
    }

    pub fn pure<T: Serialize>(&mut self, value: &T) -> Argument {
        let bytes = bcs::to_bytes(value).unwrap_or_else(|error| {
            self.error.get_or_insert(format!("Invalid Sui pure input: {error}"));
            vec![]
        });
        match self.pure_inputs.get(&bytes) {
            Some(argument) => *argument,
            None => {
                let argument = self.pure_bytes(bytes.clone());
                self.pure_inputs.insert(bytes, argument);
                argument
            }
        }
    }

    pub fn pure_bytes(&mut self, bytes: Vec<u8>) -> Argument {
        self.input(Input::Pure(bytes))
    }

    pub fn object(&mut self, object: ObjectInput) -> Argument {
        let Some(index) = self.object_inputs.get(&object.object_id()).copied() else {
            self.object_inputs.insert(object.object_id(), self.inputs.len());
            return self.input(Input::Object(object));
        };
        if let (Some(Input::Object(ObjectInput::Shared(existing))), ObjectInput::Shared(shared)) = (self.inputs.get_mut(index), &object)
            && shared.mutability == Mutability::Mutable
        {
            existing.mutability = Mutability::Mutable;
        }
        Argument::Input(index as u16)
    }

    pub fn funds_withdrawal(&mut self, coin_type: TypeTag, amount: u64) -> Argument {
        self.input(Input::FundsWithdrawal(FundsWithdrawal {
            reservation: WithdrawalReservation::MaxAmount(amount),
            type_arg: WithdrawalType::Balance(coin_type),
            withdraw_from: WithdrawFrom::Sender,
        }))
    }

    pub fn move_call(&mut self, move_call: MoveCall) -> Argument {
        self.command(Command::MoveCall(move_call))
    }

    pub fn transfer_objects(&mut self, objects: Vec<Argument>, address: Argument) {
        self.command(Command::TransferObjects(TransferObjects { objects, address }));
    }

    pub fn split_coins(&mut self, coin: Argument, amounts: Vec<Argument>) -> Vec<Argument> {
        let count = u16::try_from(amounts.len()).unwrap_or_else(|_| {
            self.error.get_or_insert("Sui split has too many amounts".to_string());
            0
        });
        self.command(Command::SplitCoins(SplitCoins { coin, amounts })).to_nested(count)
    }

    pub fn merge_coins(&mut self, coin: Argument, coins_to_merge: Vec<Argument>) {
        self.command(Command::MergeCoins(MergeCoins { coin, coins_to_merge }));
    }

    pub fn make_move_vec(&mut self, element_type: Option<TypeTag>, elements: Vec<Argument>) -> Argument {
        self.command(Command::MakeMoveVector(MakeMoveVector { element_type, elements }))
    }

    pub fn build(self) -> Result<ProgrammableTransaction, SuiError> {
        if let Some(error) = self.error {
            return Err(SuiError::invalid_input(error));
        }
        if self.inputs.len() > MAX_ARGUMENTS || self.commands.len() > MAX_ARGUMENTS {
            return Err(SuiError::invalid_input("Sui transaction has too many inputs or commands"));
        }
        Ok(ProgrammableTransaction {
            inputs: self.inputs,
            commands: self.commands,
        })
    }

    fn input(&mut self, input: Input) -> Argument {
        self.inputs.push(input);
        Argument::Input((self.inputs.len() - 1) as u16)
    }

    fn command(&mut self, command: Command) -> Argument {
        self.commands.push(command);
        Argument::Result((self.commands.len() - 1) as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Digest;

    #[test]
    fn test_build_deduplicates_inputs() {
        let mut txb = TransactionBuilder::new();
        let gas = txb.gas();
        let amount = txb.pure(&1u64);
        let same_amount = txb.pure(&1u64);
        let unique_amount = txb.pure_bytes(bcs::to_bytes(&1u64).unwrap());
        let clock = txb.object(ObjectInput::shared(Address::ZERO, 1, false));
        let mutable_clock = txb.object(ObjectInput::shared(Address::ZERO, 9, true));
        let owned_clock = txb.object(ObjectInput::immutable_or_owned(Address::ZERO, 2, Digest::ZERO));
        let coins = txb.split_coins(gas, vec![amount, same_amount, unique_amount]);
        txb.merge_coins(gas, vec![coins[2], clock, mutable_clock, owned_clock]);

        assert_eq!(txb.build().unwrap(), ProgrammableTransaction::mock());
    }

    #[test]
    fn test_build_rejects_invalid_pure_input() {
        let mut txb = TransactionBuilder::new();
        txb.pure(&f64::MAX);

        assert!(txb.build().is_err());
    }

    #[test]
    fn test_build_rejects_too_many_split_amounts() {
        let mut txb = TransactionBuilder::new();
        let amount = txb.pure(&1u64);
        let coins = txb.split_coins(Argument::Gas, vec![amount; u16::MAX as usize]);

        assert_eq!(coins.len(), u16::MAX as usize);
        assert_eq!(coins.last(), Some(&Argument::NestedResult(0, u16::MAX - 1)));
        assert!(txb.build().is_ok());

        let mut txb = TransactionBuilder::new();
        let amount = txb.pure(&1u64);

        assert!(txb.split_coins(Argument::Gas, vec![amount; u16::MAX as usize + 1]).is_empty());
        assert!(txb.build().is_err());
    }
}
