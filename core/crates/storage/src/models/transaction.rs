use chrono::NaiveDateTime;
use diesel::prelude::*;
use num_bigint::BigUint;
use primitives::{Chain, Transaction, TransactionDirection, TransactionId, TransactionUtxoInput};
use serde::de::Error as _;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::sql_types::{AssetId, ChainRow, TransactionState, TransactionType};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionRow {
    pub id: i64,
    pub chain: ChainRow,
    pub hash: String,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub memo: Option<String>,
    pub state: TransactionState,
    pub kind: TransactionType,
    pub value: Option<String>,
    pub asset_id: AssetId,
    pub fee: Option<String>,
    pub utxo_inputs: Option<serde_json::Value>,
    pub utxo_outputs: Option<serde_json::Value>,
    pub fee_asset_id: AssetId,
    pub metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionRow {
    pub chain: ChainRow,
    pub hash: String,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub memo: Option<String>,
    pub state: TransactionState,
    pub kind: TransactionType,
    pub value: Option<String>,
    pub asset_id: AssetId,
    pub fee: Option<String>,
    pub utxo_inputs: Option<serde_json::Value>,
    pub utxo_outputs: Option<serde_json::Value>,
    pub fee_asset_id: AssetId,
    pub metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
}

impl TransactionRow {
    pub fn chain(&self) -> Chain {
        self.chain.0
    }

    pub fn get_addresses(&self) -> Vec<String> {
        vec![self.from_address.clone(), self.to_address.clone()].into_iter().flatten().collect()
    }

    pub fn as_primitive(&self, addresses: Vec<String>) -> Result<Transaction, serde_json::Error> {
        let chain = self.chain();
        let transaction_id = TransactionId::new(chain, self.hash.clone());
        let asset_id = self.asset_id.0.clone();
        let from = self.from_address.clone().unwrap_or_default();
        let to_address = self.to_address.clone().unwrap_or_default();
        let inputs: Option<Vec<TransactionUtxoInput>> = serde_json::from_value(self.utxo_inputs.clone().into())?;
        let outputs: Option<Vec<TransactionUtxoInput>> = serde_json::from_value(self.utxo_outputs.clone().into())?;

        let direction = TransactionDirection::from_parties(&from, &to_address, &addresses);
        let transaction_type = self.kind.0.clone();
        let fee = BigUint::from_str(self.fee.as_deref().unwrap_or("0")).map_err(serde_json::Error::custom)?;
        let value = BigUint::from_str(self.value.as_deref().unwrap_or("0")).map_err(serde_json::Error::custom)?;

        Ok(Transaction {
            id: transaction_id.clone(),
            asset_id,
            from: from.clone(),
            to: to_address.clone(),
            contract: None,
            transaction_type,
            state: self.state.0,
            block_number: None,
            sequence: None,
            fee,
            fee_asset_id: self.fee_asset_id.0.clone(),
            value,
            memo: self.memo.clone(),
            direction,
            utxo_inputs: inputs.unwrap_or_default().into(),
            utxo_outputs: outputs.unwrap_or_default().into(),
            metadata: self.metadata.clone(),
            data: None,
            created_at: self.created_at.and_utc(),
        })
    }
}

impl NewTransactionRow {
    pub fn from_primitive(transaction: Transaction) -> Self {
        let utxo_inputs = if transaction.utxo_inputs.clone().unwrap_or_default().is_empty() {
            None
        } else {
            serde_json::to_value(transaction.utxo_inputs.clone()).ok()
        };
        let utxo_outputs = if transaction.utxo_outputs.clone().unwrap_or_default().is_empty() {
            None
        } else {
            serde_json::to_value(transaction.utxo_outputs.clone()).ok()
        };
        let metadata = if transaction.metadata.is_none() { None } else { serde_json::to_value(transaction.metadata.clone()).ok() };
        let hash = transaction.hash().to_string();
        let from_address = if transaction.from.is_empty() { None } else { Some(transaction.from) };
        let to_address = if transaction.to.is_empty() { None } else { Some(transaction.to) };
        let memo = transaction.memo.map(|memo| memo.replace('\0', "")).filter(|memo| !memo.is_empty());
        let value = if transaction.value == BigUint::ZERO { None } else { Some(transaction.value.to_string()) };

        Self {
            chain: transaction.asset_id.chain.into(),
            hash,
            memo,
            asset_id: transaction.asset_id.into(),
            value,
            fee: Some(transaction.fee.to_string()),
            fee_asset_id: transaction.fee_asset_id.into(),
            from_address,
            to_address,
            kind: transaction.transaction_type.into(),
            state: transaction.state.into(),
            utxo_inputs,
            utxo_outputs,
            metadata,
            created_at: transaction.created_at.naive_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NewTransactionRow, TransactionRow};
    use crate::sql_types::{TransactionState, TransactionType};
    use primitives::{AssetId, Chain, Transaction, TransactionDirection};

    #[test]
    fn test_as_primitive() {
        let row = TransactionRow {
            id: 1,
            chain: Chain::Ethereum.into(),
            hash: "0x0a1a78e51d162d08f73c8ea1cab5967076d15e6a2defd998c97b476c88c88b48".to_string(),
            from_address: Some("0x54914A963c4197172130C26D496a367bD6609D88".to_string()),
            to_address: Some("0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".to_string()),
            memo: None,
            state: TransactionState::Confirmed,
            kind: TransactionType::Transfer,
            value: Some("28603917".to_string()),
            asset_id: AssetId::from_token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").into(),
            fee: Some("1".to_string()),
            utxo_inputs: None,
            utxo_outputs: None,
            fee_asset_id: AssetId::from_chain(Chain::Ethereum).into(),
            metadata: None,
            created_at: chrono::Utc::now().naive_utc(),
        };
        let user = "0x0d9dab1a248f63b0a48965ba8435e4de7497a3dc".to_string();

        assert_eq!(row.as_primitive(vec![user]).unwrap().direction, TransactionDirection::Incoming);
        assert_eq!(row.as_primitive(vec![row.from_address.clone().unwrap()]).unwrap().direction, TransactionDirection::Outgoing);
        assert_eq!(
            row.as_primitive(vec![row.from_address.clone().unwrap(), row.to_address.clone().unwrap()]).unwrap().direction,
            TransactionDirection::SelfTransfer
        );
        assert_eq!(row.as_primitive(vec![]).unwrap().direction, TransactionDirection::SelfTransfer);
    }

    #[test]
    fn test_from_primitive_strips_nul_from_memo() {
        let stripped = NewTransactionRow::from_primitive(Transaction {
            memo: Some("a\0b".to_string()),
            ..Transaction::mock()
        });
        assert_eq!(stripped.memo.as_deref(), Some("ab"));

        let only_nul = NewTransactionRow::from_primitive(Transaction {
            memo: Some("\0".to_string()),
            ..Transaction::mock()
        });
        assert_eq!(only_nul.memo, None);
    }
}
