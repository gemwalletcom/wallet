use super::{
    model::TransactionInput,
    replay::{TransactionJsonReplay, parse_transaction_json},
    spend::WalletObject,
};
use crate::{address::SuiAddress, full_coin_type, types::ObjectInput};

impl TransactionJsonReplay {
    pub fn mock(transaction_json: &str) -> Self {
        Self::mock_with_wallet_coins(transaction_json, &[])
    }

    pub fn mock_with_wallet_coins(transaction_json: &str, wallet_coins: &[(usize, &str)]) -> Self {
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
        Self {
            transaction,
            object_inputs,
            wallet_objects: wallet_coins.iter().map(|(index, coin_type)| (*index, WalletObject::Coin(full_coin_type(coin_type)))).collect(),
        }
    }
}
