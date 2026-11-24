use num_bigint::BigUint;
use std::error::Error;

use crate::constants::{TRANSACTION_TYPE_ASSET_TRANSFER, TRANSACTION_TYPE_PAY};
use crate::models::{AssetTransferTransaction, Block, BlockTransaction, PaymentTransaction, Transaction as AlgoTransaction, TransactionBroadcast, TransactionLookup};
use primitives::{AssetId, DateTime, Transaction, TransactionState, TransactionType, chain::Chain};

pub fn map_transaction_broadcast(result: &TransactionBroadcast) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(message) = &result.message {
        Err(message.clone().into())
    } else if let Some(hash) = &result.tx_id {
        Ok(hash.clone())
    } else {
        Err("Broadcast failed without specific error".into())
    }
}

pub fn map_transactions(transactions: Vec<AlgoTransaction>) -> Vec<Transaction> {
    transactions.into_iter().flat_map(map_transaction).collect::<Vec<Transaction>>()
}

pub fn map_block_transactions(block: Block, ids: Vec<String>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
    let Block { timestamp, transactions } = block;
    if ids.len() != transactions.len() {
        return Err(format!("Algorand block has {} transactions and {} ids", transactions.len(), ids.len()).into());
    }
    Ok(map_transactions(ids.into_iter().zip(transactions).map(|(id, signed)| map_block_transaction(id, timestamp, signed.transaction)).collect()))
}

fn map_block_transaction(id: String, round_time: DateTime, transaction: BlockTransaction) -> AlgoTransaction {
    AlgoTransaction {
        id,
        round_time,
        fee: transaction.fee,
        sender: Some(transaction.sender),
        note: transaction.note,
        payment_transaction: Some(PaymentTransaction {
            amount: transaction.amount,
            receiver: transaction.receiver,
        }),
        asset_transfer_transaction: transaction.asset_id.map(|asset_id| AssetTransferTransaction {
            amount: transaction.asset_amount.unwrap_or_default(),
            asset_id,
            receiver: transaction.asset_receiver.unwrap_or_default(),
        }),
        transaction_type: transaction.transaction_type,
    }
}

pub fn map_transaction_by_hash(transaction: TransactionLookup) -> Option<Transaction> {
    map_transaction(transaction.transaction)
}

pub fn map_transaction(transaction: AlgoTransaction) -> Option<Transaction> {
    match transaction.transaction_type.as_str() {
        TRANSACTION_TYPE_PAY => {
            let payment = transaction.payment_transaction.clone()?;
            let value = BigUint::try_from(payment.amount.unwrap_or_default()).ok()?;
            map_transfer(&transaction, Chain::Algorand.as_asset_id(), payment.receiver.unwrap_or_default(), TransactionType::Transfer, value)
        }
        TRANSACTION_TYPE_ASSET_TRANSFER => {
            let asset_transfer = transaction.asset_transfer_transaction.clone()?;
            let transaction_type = transaction.asset_transfer_type()?;
            let asset_id = AssetId::token(Chain::Algorand, asset_transfer.asset_id.to_string());
            map_transfer(&transaction, asset_id, asset_transfer.receiver, transaction_type, BigUint::from(asset_transfer.amount))
        }
        _ => None,
    }
}

fn map_transfer(transaction: &AlgoTransaction, asset_id: AssetId, to: String, transaction_type: TransactionType, value: BigUint) -> Option<Transaction> {
    Some(Transaction::new(
        transaction.id.clone(),
        asset_id,
        transaction.sender.clone().unwrap_or_default(),
        to,
        None,
        transaction_type,
        TransactionState::Confirmed,
        BigUint::try_from(transaction.fee.unwrap_or_default()).ok()?,
        Chain::Algorand.as_asset_id(),
        value,
        transaction.get_memo(),
        None,
        transaction.round_time,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{BlockResponse, BlockTransactionIds, TransactionBroadcast, Transactions},
        provider::testkit::TEST_TRANSACTION_ID,
    };
    use primitives::DateTime;

    #[test]
    fn test_map_transaction_broadcast_success() {
        let broadcast = TransactionBroadcast {
            tx_id: Some("G4MBO3DS7ACGA3XF5XD5Y52ZVJL6ZYROTCVB2I3BQHBYHTPQ7VOA".to_string()),
            message: None,
        };
        assert_eq!(map_transaction_broadcast(&broadcast).unwrap(), "G4MBO3DS7ACGA3XF5XD5Y52ZVJL6ZYROTCVB2I3BQHBYHTPQ7VOA");
    }

    #[test]
    fn test_map_transaction_broadcast_success_data() {
        let broadcast: TransactionBroadcast = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_success.json")).unwrap();
        assert_eq!(map_transaction_broadcast(&broadcast).unwrap(), "LAEWXAG6FYFIEDAY76YQFKO46EIKEOIT4GTONUQFD6TL23XG45KQ");
    }

    #[test]
    fn test_map_transaction_broadcast_error_data() {
        let broadcast: TransactionBroadcast = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_error.json")).unwrap();
        assert_eq!(map_transaction_broadcast(&broadcast).unwrap_err().to_string(), "txgroup had 0 in fees, which is less than the minimum 1 * 1000");
    }

    #[test]
    fn test_map_block_transactions() {
        let block: BlockResponse = serde_json::from_str(include_str!("../../testdata/block.json")).unwrap();
        let ids: BlockTransactionIds = serde_json::from_str(include_str!("../../testdata/block_txids.json")).unwrap();

        let transactions = map_block_transactions(block.block.clone(), ids.transaction_ids).unwrap();

        assert_eq!(
            transactions
                .iter()
                .map(|transaction| (transaction.hash(), transaction.asset_id.to_string(), transaction.transaction_type.clone(), transaction.value.clone()))
                .collect::<Vec<_>>(),
            [
                ("QQXLN7C6BMXAZQQ56BDYDGDNSB7D7IGEL5KYKCTQT5TNTT2TN7XQ", "algorand_31566704", TransactionType::Transfer, 2758u64),
                ("R67WMJRYW7Q6US6BXLJWBBGC2KE6LC2AQPQHBLV55D5DEXEOZQTQ", "algorand", TransactionType::Transfer, 0),
                ("VNXKBGF7LNZX5WM5QE57FBGOI56LVDDG3VZMBY2F5X6CFT6OYEEA", "algorand_127746157", TransactionType::AssetActivation, 0),
            ]
            .map(|(hash, asset_id, transaction_type, value)| (hash, asset_id.to_string(), transaction_type, BigUint::from(value)))
            .to_vec()
        );
        assert_eq!(transactions[0].created_at, DateTime::from_timestamp(1795491063));
        assert_eq!(map_block_transactions(block.block, vec![]).unwrap_err().to_string(), "Algorand block has 4 transactions and 0 ids");
    }

    #[test]
    fn test_map_transaction_by_hash() {
        let lookup: TransactionLookup = serde_json::from_str(include_str!("../../testdata/transaction_by_hash.json")).unwrap();
        let transaction = map_transaction_by_hash(lookup).unwrap();

        assert_eq!(transaction.hash(), TEST_TRANSACTION_ID);
        assert_eq!(transaction.from, "RXIOUIR5IGFZMIZ7CR7FJXDYY4JI7NZG5UCWCZZNWXUPFJRLG6K6X5ITXM");
        assert_eq!(transaction.to, "NXSHXB3CLKPZ4JJ3LIXOKOEAB575EDDHCUTDYAKYRXZWVJ6CCQUP55ZEPE");
        assert_eq!(transaction.value, BigUint::from(100000u64));
    }

    #[test]
    fn test_map_transactions() {
        let transactions: Transactions = serde_json::from_str(include_str!("../../testdata/transactions_asset_transfer.json")).unwrap();
        let transactions = map_transactions(transactions.transactions);

        assert_eq!(
            transactions.iter().map(|transaction| (transaction.transaction_type.clone(), transaction.value.clone())).collect::<Vec<_>>(),
            vec![(TransactionType::Transfer, BigUint::from(67054139u64)), (TransactionType::AssetActivation, BigUint::ZERO)]
        );
        assert_eq!(
            transactions[0],
            Transaction::new(
                "HEXAEW76JHUPVMUF72DWNXG2FO7OVMAO3DK6RIEEGQTHKWNTA6AA".to_string(),
                AssetId::token(Chain::Algorand, "31566704"),
                "NZXX6SALQ6FVBMFMNY76E4IHO4S47QOHMO5U3BDYJ3UVVUN4J6IBE65MN4".to_string(),
                "76LLFZPDOPRADXS4VSCQNNJYCPZM7TCFXRW5VTEDVWBMYD47CHNCTG3J7U".to_string(),
                None,
                TransactionType::Transfer,
                TransactionState::Confirmed,
                BigUint::from(1000u64),
                Chain::Algorand.as_asset_id(),
                BigUint::from(67054139u64),
                None,
                None,
                DateTime::from_timestamp(1795361617),
            )
        );
    }
}
