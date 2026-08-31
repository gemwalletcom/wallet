use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use serde::{Deserialize, Serialize, de};
use serde_serializers::{deserialize_biguint_from_str, deserialize_option_biguint_from_str, deserialize_u64_from_str};

use primitives::TransactionState;

use crate::constants::{RESULT_SUCCESS, XRP_DEFAULT_ASSET_DECIMALS};

fn deserialize_issued_amount<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: de::Deserializer<'de>,
{
    let amount = String::deserialize(deserializer)?;
    BigNumberFormatter::value_from_amount_biguint(&amount, XRP_DEFAULT_ASSET_DECIMALS).map_err(de::Error::custom)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerInfo {
    pub ledger_index: u64,
    #[serde(default)]
    pub validated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerData {
    pub ledger: Ledger,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountObjects {
    pub account_objects: Option<Vec<AccountObject>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountObject {
    #[serde(rename = "LowLimit")]
    pub low_limit: AccountObjectLimit,
    #[serde(rename = "HighLimit")]
    pub high_limit: AccountObjectLimit,
    #[serde(rename = "Balance")]
    pub balance: Balance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub value: String,
}

impl AccountObjectLimit {
    pub fn symbol(&self) -> Option<String> {
        let currency_bytes: Vec<u8> = hex::decode(&self.currency).ok()?;
        String::from_utf8(currency_bytes.into_iter().filter(|b| *b != 0).collect()).ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountObjectLimit {
    pub currency: String,
    pub issuer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub close_time: i64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub ledger_index: u64,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountLedger {
    pub transactions: Vec<AccountLedgerTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLedgerTransaction {
    pub hash: String,
    pub ledger_index: i64,
    pub tx_json: AccountLedgerTransactionJSON,
    #[serde(rename = "meta")]
    pub meta: TransactionMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLedgerTransactionJSON {
    #[serde(rename = "Fee")]
    #[serde(default, deserialize_with = "deserialize_option_biguint_from_str")]
    pub fee: Option<BigUint>,
    #[serde(rename = "Account")]
    pub account: Option<String>,
    #[serde(rename = "DeliverMax")]
    pub amount: Option<Amount>,
    #[serde(rename = "Destination")]
    pub destination: Option<String>,
    #[serde(rename = "TransactionType")]
    pub transaction_type: String,
    pub date: i64,
    #[serde(rename = "DestinationTag")]
    pub destination_tag: Option<i64>,
    #[serde(rename = "Memos")]
    pub memos: Option<Vec<TransactionMemo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    #[serde(rename = "Fee")]
    #[serde(default, deserialize_with = "deserialize_option_biguint_from_str")]
    pub fee: Option<BigUint>,
    #[serde(rename = "Account")]
    pub account: Option<String>,
    #[serde(rename = "Amount")]
    pub amount: Option<Amount>,
    #[serde(rename = "Destination")]
    pub destination: Option<String>,
    #[serde(rename = "TransactionType")]
    pub transaction_type: String,
    pub date: Option<i64>,
    #[serde(rename = "DestinationTag")]
    pub destination_tag: Option<i64>,
    #[serde(rename = "Memos")]
    pub memos: Option<Vec<TransactionMemo>>,
    #[serde(rename = "metaData", alias = "meta")]
    pub meta: TransactionMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMeta {
    #[serde(rename = "TransactionResult")]
    pub result: String,
    pub delivered_amount: Option<Amount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Amount {
    Null,
    Str(#[serde(deserialize_with = "deserialize_biguint_from_str")] BigUint),
    Amount(AmountCurrency),
}

impl Amount {
    pub fn as_value_string(&self) -> Option<String> {
        match self {
            Amount::Null => None,
            Amount::Str(amount) => Some(amount.to_string()),
            Amount::Amount(amount) => Some(amount.value.to_string()),
        }
    }

    pub fn token_id(&self) -> Option<String> {
        match self {
            Amount::Null => None,
            Amount::Str(_) => None,
            Amount::Amount(amount) => amount.issuer.clone().or(amount.mpt_issuance_id.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountCurrency {
    #[serde(deserialize_with = "deserialize_issued_amount")]
    pub value: BigUint,
    pub issuer: Option<String>,
    pub currency: Option<String>,
    pub mpt_issuance_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMemo {
    #[serde(rename = "Memo")]
    pub memo: TransactionMemoData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMemoData {
    #[serde(rename = "MemoData")]
    pub data: Option<String>,
}

impl TransactionMemo {
    pub fn decoded_data(&self) -> Option<String> {
        primitives::hex::decode_hex_utf8(self.memo.data.as_ref()?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    #[serde(rename = "Balance", deserialize_with = "deserialize_u64_from_str")]
    pub balance: u64,
    #[serde(rename = "Sequence")]
    pub sequence: u64,
    #[serde(rename = "OwnerCount")]
    pub owner_count: u32,
    #[serde(rename = "Account")]
    pub account: Option<String>,
    #[serde(rename = "Flags")]
    pub flags: Option<u32>,
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfoResult {
    pub account_data: Option<AccountInfo>,
    pub ledger_current_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub minimum_fee: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub median_fee: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeesResult {
    pub drops: Fee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBroadcast {
    pub accepted: Option<bool>,
    pub engine_result_message: Option<String>,
    pub hash: Option<String>,
    pub tx_json: Option<TransactionJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionJson {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    #[serde(default)]
    pub validated: bool,
    #[serde(rename = "metaData", alias = "meta")]
    pub meta: Option<TransactionMeta>,
    #[serde(rename = "Fee")]
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub fee: BigUint,
}

impl TransactionStatus {
    pub fn state(&self) -> TransactionState {
        match &self.meta {
            Some(meta) if self.validated && meta.result == RESULT_SUCCESS => TransactionState::Confirmed,
            Some(_) if self.validated => TransactionState::Failed,
            _ => TransactionState::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_object_symbol_rlusd() {
        let account_object = AccountObjectLimit {
            currency: "524C555344000000000000000000000000000000".to_string(),
            issuer: "".to_string(),
        };
        assert_eq!(account_object.symbol(), Some("RLUSD".to_string()));
    }
}
