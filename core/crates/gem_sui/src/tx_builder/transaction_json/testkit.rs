use super::{
    model::TransactionInput,
    replay::{TransactionJsonReplay, parse_transaction_json},
};
use crate::address::SuiAddress;
use sui_transaction_builder::ObjectInput;

impl TransactionJsonReplay {
    pub fn mock(transaction_json: &str) -> Self {
        let transaction = parse_transaction_json(transaction_json).unwrap();
        let object_inputs = transaction
            .inputs
            .iter()
            .enumerate()
            .filter_map(|(index, input)| match input {
                TransactionInput::UnresolvedObject { object } => Some((index, ObjectInput::shared(SuiAddress::parse(&object.object_id).unwrap().into(), 1, index % 2 == 0))),
                TransactionInput::Pure { .. } | TransactionInput::Object { .. } | TransactionInput::FundsWithdrawal { .. } | TransactionInput::UnresolvedPure { .. } => None,
            })
            .collect();
        Self { transaction, object_inputs }
    }
}
