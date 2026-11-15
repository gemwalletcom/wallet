use super::{
    builder::{CommandOutput, output_argument, replay_command, replay_input},
    model::{TransactionArgument, TransactionBuilderJson},
    resolver::{input_mutability, object_inputs},
};
use crate::{
    SuiClient, SuiError,
    tx_builder::{Argument, TransactionBuilder, build_sui_gas_coin},
    types::ObjectInput,
};
use std::collections::HashMap;

pub struct ReplayedTransaction {
    pub txb: TransactionBuilder,
    pub address_balance: u64,
    outputs: Vec<CommandOutput>,
}

impl ReplayedTransaction {
    pub fn argument(&self, argument: &TransactionArgument) -> Result<Argument, SuiError> {
        output_argument(argument, &self.outputs)
    }
}

pub struct TransactionJsonReplay {
    pub(super) transaction: TransactionBuilderJson,
    pub(super) object_inputs: HashMap<usize, ObjectInput>,
}

impl TransactionJsonReplay {
    pub fn replay(&self) -> Result<ReplayedTransaction, SuiError> {
        self.replay_with_address_balance(0)
    }

    pub fn replay_with_address_balance(&self, address_balance: u64) -> Result<ReplayedTransaction, SuiError> {
        let mut txb = TransactionBuilder::new();
        let address_balance = self.fund_gas_coin(&mut txb, address_balance)?;
        let inputs = self
            .transaction
            .inputs
            .iter()
            .enumerate()
            .map(|(index, input)| replay_input(&mut txb, index, input, &self.object_inputs))
            .collect::<Result<Vec<_>, _>>()?;

        let mut outputs = Vec::new();
        for command in self.transaction.commands.iter().cloned() {
            let output = replay_command(&mut txb, command, &inputs, &outputs)?;
            outputs.push(output);
        }

        Ok(ReplayedTransaction { txb, address_balance, outputs })
    }

    fn fund_gas_coin(&self, txb: &mut TransactionBuilder, address_balance: u64) -> Result<u64, SuiError> {
        if self.transaction.withdraws_sui() {
            return Ok(0);
        }
        if !self.transaction.uses_gas_coin() {
            return Ok(address_balance);
        }
        build_sui_gas_coin(txb, address_balance)?;
        Ok(0)
    }
}

pub async fn prepare_transaction_json_replay(client: &SuiClient, transaction_json: &str) -> Result<TransactionJsonReplay, SuiError> {
    prepare_replay(client, parse_transaction_json(transaction_json)?).await
}

pub(super) fn parse_transaction_json(transaction_json: &str) -> Result<TransactionBuilderJson, SuiError> {
    let transaction: TransactionBuilderJson = serde_json::from_str(transaction_json).map_err(|err| SuiError::invalid_input(format!("Invalid Sui transaction JSON: {err}")))?;
    if transaction.version != 2 {
        return Err(SuiError::invalid_input(format!("Unsupported Sui transaction JSON version {}", transaction.version)));
    }
    Ok(transaction)
}

pub(super) async fn prepare_replay(client: &SuiClient, transaction: TransactionBuilderJson) -> Result<TransactionJsonReplay, SuiError> {
    let input_mutability = input_mutability(client, &transaction.commands).await?;
    let object_inputs = object_inputs(client, &transaction.inputs, &input_mutability).await?;
    Ok(TransactionJsonReplay { transaction, object_inputs })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SUI_COIN_TYPE,
        models::Coin,
        sui_framework_package_address,
        tx_builder::{TransactionBuilderInput, finish_transaction},
        types::{Command, FundsWithdrawal, Identifier, Input, MergeCoins, MoveCall, SplitCoins, WithdrawFrom, WithdrawalReservation, WithdrawalType},
    };
    use primitives::hex;

    #[test]
    fn test_replay() {
        let input = TransactionBuilderInput::new("0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1", 750, 50_000_000, vec![Coin::mock_sui().to_input()]);
        let replayed = |transaction_json: &str| {
            let replay = TransactionJsonReplay::mock(transaction_json);
            hex::encode(finish_transaction(replay.replay().unwrap().txb, input.clone()).unwrap().hash)
        };

        assert_eq!(replayed(include_str!("../../../testdata/transaction_builder_json.json")), "37dada8b1dc25a4578a07ac6e570efd58394606f38ab890662f83f344cff5993");
        assert_eq!(replayed(include_str!("../../../testdata/wc_sign_transaction_cetus.json")), "fdbe2dccd4f018def50c283db7bc100ccc3f96b8c6bd9a7032d9cd4feddbe96e");
        assert_eq!(
            replayed(include_str!("../../../testdata/wc_sign_transaction_cetus_funds_withdrawal.json")),
            "2a5bfd60344c7e65e228e08bbf762b88871e18496cbc8f6b5ad3a697ddfdf71c"
        );
    }

    #[test]
    fn test_replay_with_address_balance() {
        let address_balance = 626_400_000_000;
        let replayed = |replay: &TransactionJsonReplay, address_balance: u64| {
            let replayed = replay.replay_with_address_balance(address_balance).unwrap();
            (replayed.address_balance, replayed.txb.build().unwrap())
        };

        let split_gas = TransactionJsonReplay::mock(include_str!("../../../testdata/transaction_builder_json.json"));
        let funded = split_gas.replay_with_address_balance(address_balance).unwrap();
        assert_eq!(funded.address_balance, 0);
        assert_eq!(funded.argument(&TransactionArgument::NestedResult { nested_result: [0, 0] }).unwrap(), Argument::NestedResult(2, 0));
        let ptb = funded.txb.build().unwrap();
        assert_eq!(
            ptb.inputs[0],
            Input::FundsWithdrawal(FundsWithdrawal {
                reservation: WithdrawalReservation::MaxAmount(address_balance),
                type_arg: WithdrawalType::Balance(SUI_COIN_TYPE.parse().unwrap()),
                withdraw_from: WithdrawFrom::Sender,
            })
        );
        assert_eq!(
            ptb.commands[..3],
            [
                Command::MoveCall(MoveCall {
                    package: sui_framework_package_address(),
                    module: Identifier::new("coin").unwrap(),
                    function: Identifier::new("redeem_funds").unwrap(),
                    type_arguments: vec![SUI_COIN_TYPE.parse().unwrap()],
                    arguments: vec![Argument::Input(0)],
                }),
                Command::MergeCoins(MergeCoins {
                    coin: Argument::Gas,
                    coins_to_merge: vec![Argument::Result(0)],
                }),
                Command::SplitCoins(SplitCoins {
                    coin: Argument::Gas,
                    amounts: vec![Argument::Input(1)],
                }),
            ]
        );
        assert_eq!(replayed(&split_gas, 0).1.commands.len(), 2);

        let no_gas_coin = TransactionJsonReplay::mock(include_str!("../../../testdata/wc_sign_transaction_cetus.json"));
        let sui_withdrawn = TransactionJsonReplay::mock(
            r#"{"version":2,"inputs":[{"Pure":{"bytes":"AQAAAAAAAAA="}},{"FundsWithdrawal":{"reservation":{"MaxAmountU64":"1"},"typeArg":{"Balance":"0x2::sui::SUI"},"withdrawFrom":{"Sender":true}}}],"commands":[{"SplitCoins":{"coin":{"GasCoin":true},"amounts":[{"Input":0}]}},{"MoveCall":{"package":"0x2","module":"coin","function":"redeem_funds","typeArguments":["0x2::sui::SUI"],"arguments":[{"Input":1}]}}]}"#,
        );
        assert_eq!(replayed(&no_gas_coin, address_balance), (address_balance, replayed(&no_gas_coin, 0).1));
        assert_eq!(replayed(&sui_withdrawn, address_balance), (0, replayed(&sui_withdrawn, 0).1));
    }
}
