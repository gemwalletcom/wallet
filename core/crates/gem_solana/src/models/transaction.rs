use num_bigint::BigUint;
use primitives::{AssetId, Chain};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::UInt64;
use crate::models::token::{BigInt, TokenBalance, TokenBalanceChange};

#[derive(Deserialize)]
pub struct SolanaTransaction {
    pub meta: SolanaTransactionMeta,
    pub slot: UInt64,
}

#[derive(Deserialize)]
pub struct SolanaTransactionMeta {
    err: Option<serde_json::Value>,
}

impl SolanaTransactionMeta {
    pub fn has_error(&self) -> bool {
        self.err.is_some()
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub err: Option<serde_json::Value>,
    pub fee: u64,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub pre_token_balances: Vec<TokenBalance>,
    pub post_token_balances: Vec<TokenBalance>,
    pub loaded_addresses: Option<LoadedAddresses>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoadedAddresses {
    pub writable: Vec<String>,
    pub readonly: Vec<String>,
}

impl Meta {
    pub fn has_error(&self) -> bool {
        self.err.is_some()
    }

    pub fn get_pre_token_balance(&self, account_index: i64) -> Option<TokenBalance> {
        self.pre_token_balances.iter().find(|b| b.account_index == account_index).cloned()
    }

    pub fn get_post_token_balance(&self, account_index: i64) -> Option<TokenBalance> {
        self.post_token_balances.iter().find(|b| b.account_index == account_index).cloned()
    }

    pub fn get_token_balance_changes_by_owner(&self, owner: &str) -> Vec<TokenBalanceChange> {
        let mut deltas: HashMap<String, BigInt> = HashMap::new();
        for balance in self.post_token_balances.iter().filter(|balance| balance.owner == owner) {
            *deltas.entry(balance.mint.clone()).or_default() += BigInt::from(balance.get_amount());
        }
        for balance in self.pre_token_balances.iter().filter(|balance| balance.owner == owner) {
            *deltas.entry(balance.mint.clone()).or_default() -= BigInt::from(balance.get_amount());
        }

        let mut changes: Vec<TokenBalanceChange> = deltas
            .into_iter()
            .filter(|(_, amount)| *amount != BigInt::from(0))
            .map(|(mint, amount)| TokenBalanceChange {
                asset_id: AssetId::from_token(Chain::Solana, &mint),
                amount,
            })
            .collect();
        changes.sort_by_key(|change| change.asset_id.to_string());
        changes
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub message: TransactionMessage,
    pub signatures: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub block_time: i64,
    pub signature: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    pub program_id_index: usize,
    #[serde(default)]
    pub accounts: Vec<u8>,
    #[serde(default)]
    pub data: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionMessage {
    pub account_keys: Vec<String>,
    #[serde(default)]
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransaction {
    pub meta: Meta,
    pub transaction: Transaction,
}

impl BlockTransaction {
    pub fn account_key(&self, index: usize) -> Option<&String> {
        self.transaction
            .message
            .account_keys
            .iter()
            .chain(self.meta.loaded_addresses.iter().flat_map(|addresses| addresses.writable.iter().chain(&addresses.readonly)))
            .nth(index)
    }

    pub fn fee(&self) -> BigUint {
        BigUint::from(self.meta.fee)
    }

    pub fn get_balance_change(&self, address: &str) -> u64 {
        let index = self.transaction.message.account_keys.iter().position(|k| k == address);
        match index {
            Some(i) => {
                let pre = self.meta.pre_balances.get(i).copied().unwrap_or(0);
                let post = self.meta.post_balances.get(i).copied().unwrap_or(0);
                pre.saturating_sub(post).saturating_sub(self.meta.fee)
            }
            None => 0,
        }
    }

    pub fn get_balance_changes_by_owner(&self, owner: &str) -> TokenBalanceChange {
        // Find all account indices that belong to the owner
        let account_indices: Vec<usize> = self.transaction.message.account_keys.iter().enumerate().filter_map(|(i, k)| if k == owner { Some(i) } else { None }).collect();

        let (total_pre, total_post) = account_indices.into_iter().fold((0u64, 0u64), |(pre_acc, post_acc), idx| {
            let pre = *self.meta.pre_balances.get(idx).unwrap_or(&0);
            let post = *self.meta.post_balances.get(idx).unwrap_or(&0);
            (pre_acc.wrapping_add(pre), post_acc.wrapping_add(post))
        });

        let delta = BigInt::from(total_post) - BigInt::from(total_pre);
        let amount = match self.is_fee_payer(owner) {
            true => delta + BigInt::from(self.meta.fee),
            false => delta,
        };

        TokenBalanceChange {
            asset_id: Chain::Solana.as_asset_id(),
            amount,
        }
    }

    fn is_fee_payer(&self, owner: &str) -> bool {
        self.transaction.message.account_keys.first().is_some_and(|key| key == owner)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransactions {
    pub block_time: i64,
    pub transactions: Vec<BlockTransaction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleTransaction {
    pub block_time: i64,
    pub meta: Meta,
    pub slot: u64,
    pub transaction: Transaction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_one_transaction_parses_without_lookup_addresses() {
        let json = r#"{
            "meta": {
                "err": null,
                "fee": 5000,
                "preBalances": [100000, 0],
                "postBalances": [85000, 10000],
                "preTokenBalances": [],
                "postTokenBalances": []
            },
            "version": 1,
            "transaction": {
                "message": {
                    "accountKeys": ["sender", "program"],
                    "instructions": [],
                    "transactionConfig": {"computeUnitLimit": 200000, "loadedAccountsDataSizePages": 1}
                },
                "signatures": ["signature"]
            }
        }"#;

        let transaction: BlockTransaction = serde_json::from_str(json).unwrap();

        assert_eq!(transaction.account_key(0).map(String::as_str), Some("sender"));
        assert_eq!(transaction.get_balance_change("sender"), 10000);
    }

    #[test]
    fn test_account_key() {
        let mut transaction = BlockTransaction::mock(&["sender", "program"], vec![], vec![]);
        assert_eq!(transaction.account_key(0).map(String::as_str), Some("sender"));
        assert_eq!(transaction.account_key(2), None);

        transaction.meta.loaded_addresses = Some(LoadedAddresses {
            writable: vec!["destination".to_string()],
            readonly: vec!["mint".to_string()],
        });

        assert_eq!(
            (0..5).map(|index| transaction.account_key(index).map(String::as_str)).collect::<Vec<_>>(),
            vec![Some("sender"), Some("program"), Some("destination"), Some("mint"), None]
        );
    }

    #[test]
    fn test_balance_change() {
        let tx = BlockTransaction::mock(&["sender", "recipient"], vec![100_000, 0], vec![85_000, 10_000]);
        assert_eq!(tx.get_balance_change("sender"), 10_000);
    }

    #[test]
    fn test_balance_change_no_change() {
        let tx = BlockTransaction::mock(&["sender"], vec![100_000], vec![95_000]);
        assert_eq!(tx.get_balance_change("sender"), 0);
    }

    #[test]
    fn test_balance_change_received() {
        let tx = BlockTransaction::mock(&["sender"], vec![100_000], vec![200_000]);
        assert_eq!(tx.get_balance_change("sender"), 0);
    }

    #[test]
    fn test_token_accounts_sharing_a_mint_are_summed_into_one_change() {
        let mint = "So11111111111111111111111111111111111111112";
        let other = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
        let mut transaction = BlockTransaction::mock(&["owner"], vec![0], vec![0]);
        transaction.meta.pre_token_balances = vec![
            TokenBalance::mock(mint, "owner", 300),
            TokenBalance::mock(other, "owner", 50),
            TokenBalance::mock(mint, "owner", 700),
            TokenBalance::mock(mint, "someone-else", 9_000),
        ];
        transaction.meta.post_token_balances = vec![TokenBalance::mock(mint, "owner", 250)];

        let changes = transaction.meta.get_token_balance_changes_by_owner("owner");

        assert_eq!(
            changes,
            vec![
                TokenBalanceChange {
                    asset_id: AssetId::from_token(Chain::Solana, other),
                    amount: BigInt::from(-50),
                },
                TokenBalanceChange {
                    asset_id: AssetId::from_token(Chain::Solana, mint),
                    amount: BigInt::from(-750),
                },
            ],
            "every account of a mint counts, and a closed one counts as zero"
        );

        transaction.meta.pre_token_balances.reverse();
        assert_eq!(
            changes,
            transaction.meta.get_token_balance_changes_by_owner("owner"),
            "the order the node listed the accounts in does not change the net amount"
        );
    }

    #[test]
    fn test_the_owner_balance_change_restores_the_fee_only_for_the_account_that_paid_it() {
        let sent = BlockTransaction::mock(&["sender", "recipient"], vec![100_000, 0], vec![85_000, 10_000]);
        assert_eq!(sent.get_balance_changes_by_owner("sender").amount, BigInt::from(-10_000), "the payer sent the transfer, not the transfer plus its fee");

        let received = BlockTransaction::mock(&["receiver"], vec![100_000], vec![200_000]);
        assert_eq!(received.get_balance_changes_by_owner("receiver").amount, BigInt::from(105_000), "a payer who receives already paid the fee out of what arrived");

        let sponsored = BlockTransaction::mock(&["payer", "owner"], vec![100_000, 0], vec![90_000, 5_000]);
        assert_eq!(sponsored.get_balance_changes_by_owner("owner").amount, BigInt::from(5_000), "an account that paid no fee has none to restore");
        assert_eq!(sponsored.get_balance_changes_by_owner("unknown").amount, BigInt::from(0));
    }

    #[test]
    fn test_balance_change_unknown_address() {
        let tx = BlockTransaction::mock(&["sender"], vec![100_000], vec![85_000]);
        assert_eq!(tx.get_balance_change("unknown"), 0);
    }
}
