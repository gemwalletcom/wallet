use crate::models::rpc::{AccountLedger, AccountLedgerTransaction, Amount, Ledger, RawTransactionEntry, Transaction as XrpTransaction, TransactionBroadcast, TransactionMemo, TransactionMeta, memos_hash};
use crate::{RESULT_SUCCESS, TRANSACTION_TYPE_BATCH, TRANSACTION_TYPE_PAYMENT, XRP_EPOCH_OFFSET_SECONDS};
use chrono::DateTime;
use num_bigint::{BigInt, BigUint};
use primitives::{AssetId, SwapProvider, Transaction, TransactionState, TransactionSwapMetadata, TransactionSwapReferralFee, TransactionType, chain::Chain, swap::XRP_REFERRAL_ADDRESS};
use std::collections::HashSet;
use std::error::Error;

struct LedgerEntry<'a> {
    hash: &'a str,
    transaction_type: &'a str,
    account: Option<&'a str>,
    destination: Option<&'a str>,
    amount: Option<&'a Amount>,
    destination_tag: Option<i64>,
    memos: Option<&'a [TransactionMemo]>,
    fee: Option<&'a BigUint>,
    raw_transactions: Option<&'a [RawTransactionEntry]>,
    meta: &'a TransactionMeta,
    timestamp: i64,
}

impl<'a> LedgerEntry<'a> {
    fn from_transaction(transaction: &'a XrpTransaction, date: i64) -> Self {
        Self {
            hash: &transaction.hash,
            transaction_type: &transaction.transaction_type,
            account: transaction.account.as_deref(),
            destination: transaction.destination.as_deref(),
            amount: transaction.amount.as_ref(),
            destination_tag: transaction.destination_tag,
            memos: transaction.memos.as_deref(),
            fee: transaction.fee.as_ref(),
            raw_transactions: transaction.raw_transactions.as_deref(),
            meta: &transaction.meta,
            timestamp: XRP_EPOCH_OFFSET_SECONDS + date,
        }
    }

    fn from_account(transaction: &'a AccountLedgerTransaction) -> Self {
        let tx = &transaction.tx_json;
        Self {
            hash: &transaction.hash,
            transaction_type: &tx.transaction_type,
            account: tx.account.as_deref(),
            destination: tx.destination.as_deref(),
            amount: tx.amount.as_ref(),
            destination_tag: tx.destination_tag,
            memos: tx.memos.as_deref(),
            fee: tx.fee.as_ref(),
            raw_transactions: tx.raw_transactions.as_deref(),
            meta: &transaction.meta,
            timestamp: XRP_EPOCH_OFFSET_SECONDS + tx.date,
        }
    }

    fn is_swap_leg(&self) -> bool {
        (self.destination.is_some() && self.destination == self.account) || self.destination == Some(XRP_REFERRAL_ADDRESS)
    }
}

pub fn map_transaction_broadcast(broadcast_result: &TransactionBroadcast) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(accepted) = broadcast_result.accepted
        && !accepted
    {
        if let Some(error_msg) = &broadcast_result.engine_result_message {
            return Err(format!("Transaction rejected: {}", error_msg).into());
        }
        return Err("Transaction was not accepted".into());
    }

    if let Some(hash) = &broadcast_result.hash {
        Ok(hash.clone())
    } else if let Some(tx_json) = &broadcast_result.tx_json {
        Ok(tx_json.hash.clone())
    } else {
        Err("Transaction broadcast failed - no hash returned".into())
    }
}

pub fn map_transactions_by_block(ledger: Ledger) -> Vec<Transaction> {
    map_block_transactions(Chain::Xrp, ledger)
}

pub fn map_transactions_by_address(account_ledger: AccountLedger) -> Vec<Transaction> {
    map_account_transactions(Chain::Xrp, account_ledger)
}

pub fn map_block_transactions(chain: Chain, ledger: Ledger) -> Vec<Transaction> {
    map_entries(chain, &ledger.transactions.iter().map(|transaction| LedgerEntry::from_transaction(transaction, ledger.close_time)).collect::<Vec<_>>())
}

pub fn map_account_transactions(chain: Chain, ledger: AccountLedger) -> Vec<Transaction> {
    map_entries(chain, &ledger.transactions.iter().map(LedgerEntry::from_account).collect::<Vec<_>>())
}

pub fn map_direct_transaction(chain: Chain, transaction: XrpTransaction) -> Option<Transaction> {
    map_transaction_common(chain, &LedgerEntry::from_transaction(&transaction, transaction.date?))
}

pub fn batch_ledger(transaction: &XrpTransaction) -> Option<(&str, u64)> {
    match transaction.transaction_type == TRANSACTION_TYPE_BATCH {
        true => Some((transaction.account.as_deref()?, transaction.ledger_index?)),
        false => None,
    }
}

pub fn split_batch_ledger(ledger: &AccountLedger, limit: usize) -> Option<u64> {
    let oldest = ledger.transactions.last().filter(|_| ledger.transactions.len() >= limit)?;
    let is_batch = |transaction: &&AccountLedgerTransaction| transaction.tx_json.transaction_type == TRANSACTION_TYPE_BATCH || transaction.meta.parent_batch_id.is_some();
    ledger.transactions.iter().filter(|transaction| transaction.ledger_index == oldest.ledger_index).find(is_batch)?;
    u64::try_from(oldest.ledger_index).ok()
}

fn map_entries(chain: Chain, entries: &[LedgerEntry]) -> Vec<Transaction> {
    let swaps = entries.iter().filter_map(|entry| map_batch_swap(chain, entry, entries)).collect::<Vec<_>>();
    let swap_ids = swaps.iter().map(Transaction::hash).collect::<HashSet<_>>();
    let is_swap_part = |entry: &LedgerEntry| entry.is_swap_leg() && entry.meta.parent_batch_id.as_deref().is_some_and(|parent| swap_ids.contains(parent));
    let transfers = entries.iter().filter(|entry| !is_swap_part(entry)).filter_map(|entry| map_transaction_common(chain, entry)).collect::<Vec<_>>();
    transfers.into_iter().chain(swaps).collect()
}

fn map_batch_swap(chain: Chain, batch: &LedgerEntry, entries: &[LedgerEntry]) -> Option<Transaction> {
    let account = batch.account?;
    let payments = || {
        batch
            .raw_transactions
            .into_iter()
            .flatten()
            .map(|entry| &entry.transaction)
            .filter(|part| part.transaction_type == TRANSACTION_TYPE_PAYMENT && part.account == account)
    };
    let swap = payments().find(|part| part.destination.as_deref() == Some(account) && part.send_max.is_some())?;
    let referral_fee = payments().find(|part| part.destination.as_deref() == Some(XRP_REFERRAL_ADDRESS)).and_then(|part| part.amount.as_ref()).and_then(|amount| {
        Some(TransactionSwapReferralFee {
            asset_id: AssetId::from(chain, amount.token_id()),
            value: amount.value()?,
        })
    });
    let (send_max, amount) = (swap.send_max.as_ref()?, swap.amount.as_ref()?);
    let (from_asset, to_asset) = (AssetId::from(chain, send_max.token_id()), AssetId::from(chain, amount.token_id()));
    let legs = entries.iter().filter(|entry| entry.meta.is_batch_part(batch.hash) && entry.is_swap_leg()).collect::<Vec<_>>();
    let (state, from_value, to_value) = match legs.is_empty() {
        false => (
            TransactionState::Confirmed,
            (-balance_change(&legs, account, &from_asset)).to_biguint()?,
            balance_change(&legs, account, &to_asset).to_biguint()?,
        ),
        true => {
            let fee_in = |asset: &AssetId| referral_fee.as_ref().filter(|fee| fee.asset_id == *asset).map_or(BigUint::ZERO, |fee| fee.value.clone());
            let (to_value, output_fee) = (amount.value()?, fee_in(&to_asset));
            (TransactionState::Failed, send_max.value()? + fee_in(&from_asset), (to_value >= output_fee).then(|| to_value - output_fee)?)
        }
    };
    let metadata = TransactionSwapMetadata::new(from_asset.clone(), from_value.clone(), to_asset, to_value, SwapProvider::XrpLedger).with_referral_fee(referral_fee);

    Some(Transaction::new(
        batch.hash.to_string(),
        from_asset,
        account.to_string(),
        account.to_string(),
        None,
        TransactionType::Swap,
        state,
        batch.fee.cloned().unwrap_or_default(),
        chain.as_asset_id(),
        from_value,
        None,
        serde_json::to_value(metadata).ok(),
        DateTime::from_timestamp(batch.timestamp, 0)?,
    ))
}

fn balance_change(legs: &[&LedgerEntry], account: &str, asset: &AssetId) -> BigInt {
    legs.iter()
        .flat_map(|leg| leg.meta.affected_nodes.iter().flatten())
        .filter_map(|node| node.modified_node.as_ref()?.balance_change(account, asset.token_id.as_deref()))
        .sum()
}

fn map_transaction_common(chain: Chain, entry: &LedgerEntry) -> Option<Transaction> {
    if entry.transaction_type != TRANSACTION_TYPE_PAYMENT {
        return None;
    }
    let memo = entry.memos.and_then(<[_]>::first).and_then(TransactionMemo::decoded_data).or_else(|| entry.destination_tag.map(|tag| tag.to_string()));
    let (state, amount) = match entry.meta.result == RESULT_SUCCESS {
        true => (TransactionState::Confirmed, entry.meta.delivered_amount.as_ref()?),
        false => (TransactionState::Failed, entry.amount?),
    };
    Some(
        Transaction::new(
            entry.hash.to_string(),
            AssetId::from(chain, amount.token_id()),
            entry.account.unwrap_or_default().to_string(),
            entry.destination.unwrap_or_default().to_string(),
            None,
            TransactionType::Transfer,
            state,
            entry.fee.cloned().unwrap_or_default(),
            chain.as_asset_id(),
            amount.value()?,
            memo,
            None,
            DateTime::from_timestamp(entry.timestamp, 0)?,
        )
        .with_data(entry.memos.and_then(memos_hash)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rpc::{LedgerData, TransactionBroadcast, TransactionMemoData};
    use crate::provider::testkit::TEST_TRANSACTION_ID;
    use gem_jsonrpc::types::JsonRpcResult;

    const BATCH_SWAP_ACCOUNT: &str = "rnCRC2AqXhFCnb9ULHtyCNrRYXgYuLBvZF";
    const BATCH_SWAP_FEE_ACCOUNT: &str = "rJGb4etn9GSwNHYVu7dNMbdiVgzqxaTSUG";
    const BATCH_SWAP_HASH: &str = "3B925B414BF7E91EC67A3E283F374DE0BC84D6E11DF5345115B44AF7DA5FEDEA";

    #[test]
    fn test_map_batch_swaps() {
        let account_ledger = |json: &str| serde_json::from_str::<JsonRpcResult<AccountLedger>>(json).unwrap().take().unwrap();
        let other_platform = account_ledger(include_str!("../testdata/account_transactions_batch_swap.json"));
        let gem = account_ledger(&include_str!("../testdata/account_transactions_batch_swap.json").replace(BATCH_SWAP_FEE_ACCOUNT, XRP_REFERRAL_ADDRESS));
        let failed = AccountLedger {
            transactions: other_platform.transactions.iter().filter(|transaction| transaction.meta.parent_batch_id.is_none()).cloned().collect(),
        };
        let xyz = AssetId::from_token(Chain::Xrp, "rU5nUwkBUPx3MFPQSHmMnr9S9yg4g8cKLc");
        let swap = |state: TransactionState, from_value: u64, to_value: u128, referral_fee: Option<TransactionSwapReferralFee>| {
            let metadata = TransactionSwapMetadata::new(Chain::Xrp.as_asset_id(), BigUint::from(from_value), xyz.clone(), BigUint::from(to_value), SwapProvider::XrpLedger).with_referral_fee(referral_fee);
            Transaction::new(
                BATCH_SWAP_HASH.to_string(),
                Chain::Xrp.as_asset_id(),
                BATCH_SWAP_ACCOUNT.to_string(),
                BATCH_SWAP_ACCOUNT.to_string(),
                None,
                TransactionType::Swap,
                state,
                BigUint::from(60u32),
                Chain::Xrp.as_asset_id(),
                BigUint::from(from_value),
                None,
                serde_json::to_value(metadata).ok(),
                DateTime::from_timestamp(XRP_EPOCH_OFFSET_SECONDS + 846_227_231, 0).unwrap(),
            )
        };
        let rows = |transactions: &[Transaction]| transactions.iter().map(|transaction| (transaction.hash().to_string(), transaction.transaction_type.clone())).collect::<Vec<_>>();
        let gem_fee = TransactionSwapReferralFee {
            asset_id: Chain::Xrp.as_asset_id(),
            value: BigUint::from(50_000u32),
        };

        let other_platform = map_account_transactions(Chain::Xrp, other_platform);
        assert_eq!(
            rows(&other_platform),
            vec![
                ("C2CDD4152623715073A2FE5243995F3D0F9C7B793F9A591F74A3A0076C002BA8".to_string(), TransactionType::Transfer),
                (BATCH_SWAP_HASH.to_string(), TransactionType::Swap)
            ]
        );
        assert_eq!(other_platform[1], swap(TransactionState::Confirmed, 5_000_000, 881_538_098_457_100_000_000, None));
        assert_eq!(map_account_transactions(Chain::Xrp, gem), vec![swap(TransactionState::Confirmed, 5_050_000, 881_538_098_457_100_000_000, Some(gem_fee))]);
        assert_eq!(map_account_transactions(Chain::Xrp, failed), vec![swap(TransactionState::Failed, 5_000_000, 962_458_111_302_866_200_000_000, None)]);
    }

    #[test]
    fn test_split_batch_ledger() {
        let ledger = serde_json::from_str::<JsonRpcResult<AccountLedger>>(include_str!("../testdata/account_transactions_batch_swap.json")).unwrap().take().unwrap();
        let cut = AccountLedger {
            transactions: ledger.transactions[..2].to_vec(),
        };

        assert_eq!(split_batch_ledger(&cut, 2), Some(107_888_305));
        assert_eq!(split_batch_ledger(&cut, 3), None);
        assert_eq!(
            split_batch_ledger(&serde_json::from_str::<JsonRpcResult<AccountLedger>>(include_str!("../testdata/account_transactions.json")).unwrap().take().unwrap(), 1),
            None
        );
    }

    #[test]
    fn test_map_transaction_broadcast_success() {
        let broadcast = serde_json::from_str::<JsonRpcResult<TransactionBroadcast>>(include_str!("../testdata/transaction_broadcast_success.json"))
            .unwrap()
            .take()
            .unwrap();

        let result = map_transaction_broadcast(&broadcast);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "04F53F220DD1BCB7CCF279D66FFB986EA41383EFC9378CA1EBF1823D7C89264F");
    }

    #[test]
    fn test_map_transaction_broadcast_failed() {
        let broadcast = serde_json::from_str::<JsonRpcResult<TransactionBroadcast>>(include_str!("../testdata/transaction_broadcast_failed.json"))
            .unwrap()
            .take()
            .unwrap();

        let result = map_transaction_broadcast(&broadcast);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction rejected: Ledger sequence too high.");
    }

    #[test]
    fn test_map_account_transactions() {
        let ledger = serde_json::from_str::<JsonRpcResult<AccountLedger>>(include_str!("../testdata/account_transactions.json")).unwrap().take().unwrap();
        let transactions = map_account_transactions(Chain::Xrp, ledger);

        let expected_tx = Transaction::new(
            "00778C36255A48E753E7CDD3B60243D551ACD4B6ABD6765E9011D28B7566FEAB".to_string(),
            Chain::Xrp.as_asset_id(),
            "rGBpbVC11etyeGpJCAPrfS1of7SrEM2Q2c".to_string(),
            "rnZmVGX6f4pUYyS4oXYJzoLdRojQV8y297".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(11u64),
            Chain::Xrp.as_asset_id(),
            BigUint::from(1u64),
            None,
            None,
            DateTime::from_timestamp(1749150631, 0).unwrap(),
        );

        assert_eq!(transactions.first().unwrap(), &expected_tx);
    }

    #[test]
    fn test_map_account_transaction_thorchain_swap() {
        let ledger = serde_json::from_str::<JsonRpcResult<AccountLedger>>(include_str!("../testdata/account_transaction_thorchain_swap.json"))
            .unwrap()
            .take()
            .unwrap();
        let transactions = map_account_transactions(Chain::Xrp, ledger);

        let transaction = transactions.first().unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.value, BigUint::from(30000000u64));
        assert_eq!(transaction.memo.as_deref(), Some("=:b:bc1q3yw4t9xlqq9982qhvp95lgs60xrjdkmjdta0jn:0/1/0:g1:50"));
    }

    #[test]
    fn test_map_transaction_bridgers_swap() {
        let transaction = serde_json::from_str::<JsonRpcResult<XrpTransaction>>(include_str!("../testdata/transaction_bridgers_swap.json")).unwrap().take().unwrap();
        let memo_data = transaction.memos.as_ref().unwrap()[0].memo.data.clone().unwrap();
        let lowercase = TransactionMemo {
            memo: TransactionMemoData { data: Some(memo_data.to_lowercase()) },
        };

        let mapped = map_direct_transaction(Chain::Xrp, transaction).unwrap();

        assert_eq!(mapped.to, "rM27yzkCw6WA3T4g1sPaeC1kpxHUhuxRWn");
        assert_eq!(mapped.value, BigUint::from(10000000u64));
        assert_eq!(mapped.data.as_deref(), Some("a03b33418f5e120e54a64a17efe5096a60bee999e4523792ecf65022667739f9"));
        assert_eq!(memos_hash(&[lowercase]), mapped.data);
    }

    #[test]
    fn test_map_transactions_by_block() {
        let ledger = serde_json::from_str::<JsonRpcResult<LedgerData>>(include_str!("../testdata/transactions_by_block.json")).unwrap().take().unwrap();
        let transactions = map_transactions_by_block(ledger.ledger);

        assert!(!transactions.is_empty());
        for transaction in transactions {
            assert_eq!(transaction.asset_id.chain, Chain::Xrp);
            assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        }
    }

    #[test]
    fn test_map_transaction_by_hash() {
        let transaction = serde_json::from_str::<JsonRpcResult<XrpTransaction>>(include_str!("../testdata/transaction_by_hash.json")).unwrap().take().unwrap();

        let mapped = map_direct_transaction(Chain::Xrp, transaction).unwrap();

        assert_eq!(mapped.hash(), TEST_TRANSACTION_ID);
        assert_eq!(mapped.from, "rnXZ876yGEhoATQSYegtD8bg8wpA8TTX5a");
    }

    #[test]
    fn test_map_transaction_uses_delivered_amount() {
        let meta = TransactionMeta {
            result: RESULT_SUCCESS.to_string(),
            delivered_amount: Some(Amount::Str(1u32.into())),
            parent_batch_id: None,
            affected_nodes: None,
        };
        let entry = LedgerEntry {
            hash: "hash",
            transaction_type: TRANSACTION_TYPE_PAYMENT,
            account: Some("sender"),
            destination: Some("recipient"),
            amount: Some(&Amount::Str(100u32.into())),
            destination_tag: None,
            memos: None,
            fee: None,
            raw_transactions: None,
            meta: &meta,
            timestamp: 0,
        };

        assert_eq!(map_transaction_common(Chain::Xrp, &entry).unwrap().value, BigUint::from(1u64));
    }
}
