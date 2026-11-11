use super::{
    builder::{CommandOutput, output_argument, replay_command, replay_input},
    model::{TransactionArgument, TransactionBuilderJson},
    resolver::{input_mutability, object_inputs},
};
use crate::{SuiClient, SuiError};
use std::collections::HashMap;
use sui_transaction_builder::{Argument, ObjectInput, TransactionBuilder};

pub struct ReplayedTransaction {
    pub txb: TransactionBuilder,
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
        let mut txb = TransactionBuilder::new();
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

        Ok(ReplayedTransaction { txb, outputs })
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
        models::Coin,
        tx_builder::{TransactionBuilderInput, finish_transaction},
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
}
