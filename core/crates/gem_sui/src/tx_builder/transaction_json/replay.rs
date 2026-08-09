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
    object_inputs: HashMap<usize, ObjectInput>,
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
