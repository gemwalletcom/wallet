use std::error::Error;

use gem_hash::sha2::sha256;
use num_bigint::{BigInt, BigUint};
use number_formatter::BigNumberFormatter;
use serde::{Deserialize, Serialize, de};
use serde_json::Value;
use serde_serializers::{deserialize_biguint_from_str, deserialize_option_biguint_from_str, deserialize_u64_from_str};

use primitives::{TransactionState, swap::QuoteAsset};

use crate::constants::{RESULT_SUCCESS, TRANSACTION_TYPE_BATCH, XRP_DEFAULT_ASSET_DECIMALS};
use crate::currency::currency_code;
use crate::models::swap::XrpPathStep;

const ANY_AMOUNT: &str = "-1";
const LEDGER_ENTRY_ACCOUNT_ROOT: &str = "AccountRoot";
const LEDGER_ENTRY_RIPPLE_STATE: &str = "RippleState";
const CURRENT_LEDGER: &str = "current";

fn deserialize_issued_amount<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: de::Deserializer<'de>,
{
    let amount = String::deserialize(deserializer)?;
    BigNumberFormatter::value_from_amount_biguint(&amount, XRP_DEFAULT_ASSET_DECIMALS).map_err(de::Error::custom)
}

fn deserialize_balance<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: de::Deserializer<'de>,
{
    let amount = String::deserialize(deserializer)?;
    BigNumberFormatter::value_from_amount(&amount, XRP_DEFAULT_ASSET_DECIMALS).map_err(de::Error::custom)?.parse().map_err(de::Error::custom)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfoResult {
    pub info: ServerInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub network_id: Option<u64>,
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
    #[serde(deserialize_with = "deserialize_balance")]
    pub value: BigInt,
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
    #[serde(rename = "RawTransactions")]
    pub raw_transactions: Option<Vec<RawTransactionEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub ledger_index: Option<u64>,
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
    #[serde(rename = "RawTransactions")]
    pub raw_transactions: Option<Vec<RawTransactionEntry>>,
    #[serde(rename = "metaData", alias = "meta")]
    pub meta: TransactionMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTransactionEntry {
    #[serde(rename = "RawTransaction")]
    pub transaction: RawTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTransaction {
    #[serde(rename = "TransactionType")]
    pub transaction_type: String,
    #[serde(rename = "Account")]
    pub account: String,
    #[serde(rename = "Destination")]
    pub destination: Option<String>,
    #[serde(rename = "Amount")]
    pub amount: Option<Amount>,
    #[serde(rename = "SendMax")]
    pub send_max: Option<Amount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMeta {
    #[serde(rename = "TransactionResult")]
    pub result: String,
    pub delivered_amount: Option<Amount>,
    #[serde(rename = "ParentBatchID")]
    pub parent_batch_id: Option<String>,
    #[serde(rename = "AffectedNodes")]
    pub affected_nodes: Option<Vec<AffectedNode>>,
}

impl TransactionMeta {
    pub fn is_batch_part(&self, batch_id: &str) -> bool {
        self.result == RESULT_SUCCESS && self.parent_batch_id.as_deref().is_some_and(|parent| parent.eq_ignore_ascii_case(batch_id))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedNode {
    #[serde(rename = "ModifiedNode")]
    pub modified_node: Option<ModifiedNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedNode {
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,
    #[serde(rename = "FinalFields")]
    pub final_fields: Option<LedgerEntryFields>,
    #[serde(rename = "PreviousFields")]
    pub previous_fields: Option<LedgerEntryFields>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntryFields {
    #[serde(rename = "Account")]
    pub account: Option<String>,
    #[serde(rename = "Balance")]
    pub balance: Option<Value>,
    #[serde(rename = "LowLimit")]
    pub low_limit: Option<AccountObjectLimit>,
    #[serde(rename = "HighLimit")]
    pub high_limit: Option<AccountObjectLimit>,
}

impl ModifiedNode {
    pub fn balance_change(&self, account: &str, issuer: Option<&str>) -> Option<BigInt> {
        let fields = self.final_fields.as_ref()?;
        let sign = match (self.ledger_entry_type.as_str(), issuer) {
            (LEDGER_ENTRY_ACCOUNT_ROOT, None) => (fields.account.as_deref() == Some(account)).then_some(1)?,
            (LEDGER_ENTRY_RIPPLE_STATE, Some(issuer)) => match (fields.low_limit.as_ref()?.issuer.as_str(), fields.high_limit.as_ref()?.issuer.as_str()) {
                (low, high) if (low, high) == (account, issuer) => 1,
                (low, high) if (low, high) == (issuer, account) => -1,
                _ => return None,
            },
            _ => return None,
        };
        let previous = match self.previous_fields.as_ref().and_then(|previous| previous.balance.as_ref()) {
            Some(balance) => ledger_balance(balance)?,
            None => BigInt::ZERO,
        };
        Some(BigInt::from(sign) * (ledger_balance(fields.balance.as_ref()?)? - previous))
    }
}

fn ledger_balance(balance: &Value) -> Option<BigInt> {
    match balance {
        Value::String(drops) => drops.parse().ok(),
        Value::Object(issued) => BigNumberFormatter::value_from_amount(issued.get("value")?.as_str()?, XRP_DEFAULT_ASSET_DECIMALS).ok()?.parse().ok(),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Amount {
    Null,
    Str(#[serde(deserialize_with = "deserialize_biguint_from_str")] BigUint),
    Amount(AmountCurrency),
}

impl Amount {
    pub fn value(&self) -> Option<BigUint> {
        match self {
            Amount::Null => None,
            Amount::Str(amount) => Some(amount.clone()),
            Amount::Amount(amount) => Some(amount.value.clone()),
        }
    }

    pub fn token_id(&self) -> Option<String> {
        match self {
            Amount::Null => None,
            Amount::Str(_) => None,
            Amount::Amount(amount) => amount.issuer.clone().or_else(|| amount.mpt_issuance_id.clone()),
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

    pub fn data_hash(&self) -> Option<String> {
        let data = hex::decode(self.memo.data.as_ref()?).ok()?;
        Some(hex::encode(sha256(&data)))
    }
}

pub fn memos_hash(memos: &[TransactionMemo]) -> Option<String> {
    match memos {
        [memo] => memo.data_hash(),
        _ => None,
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
pub struct ValidatedAccountInfoResult {
    pub account_data: Option<AccountInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub minimum_fee: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub open_ledger_fee: u64,
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
    #[serde(rename = "TransactionType")]
    pub transaction_type: String,
    pub ledger_index: Option<u64>,
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

    pub fn applied_batch_ledger(&self) -> Option<u64> {
        self.ledger_index.filter(|_| self.transaction_type == TRANSACTION_TYPE_BATCH && self.state() == TransactionState::Confirmed)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PathFindAmount {
    Drops(String),
    Issued { currency: String, issuer: String, value: String },
}

impl PathFindAmount {
    pub fn new(asset: &QuoteAsset, value: Option<&BigUint>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        match asset.asset_id().token_id {
            None => Ok(Self::Drops(value.map_or(ANY_AMOUNT.to_string(), BigUint::to_string))),
            Some(issuer) => Ok(Self::Issued {
                currency: currency_code(&asset.symbol)?,
                issuer,
                value: match value {
                    Some(value) => BigNumberFormatter::plain_value(value, asset.decimals)?,
                    None => ANY_AMOUNT.to_string(),
                },
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathFindRequest {
    pub source_account: String,
    pub destination_account: String,
    pub destination_amount: PathFindAmount,
    pub send_max: PathFindAmount,
    pub ledger_index: &'static str,
}

impl PathFindRequest {
    pub fn new(source_account: &str, destination_account: &str, send_max: PathFindAmount, destination_amount: PathFindAmount) -> Self {
        Self {
            source_account: source_account.to_string(),
            destination_account: destination_account.to_string(),
            destination_amount,
            send_max,
            ledger_index: CURRENT_LEDGER,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathFindResult {
    pub alternatives: Vec<PathAlternative>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathAlternative {
    pub source_amount: Amount,
    pub destination_amount: Option<Amount>,
    pub paths_computed: Vec<Vec<XrpPathStep>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_object_symbol_rlusd() {
        let account_object = AccountObjectLimit {
            currency: "524C555344000000000000000000000000000000".to_string(),
            issuer: String::new(),
        };
        assert_eq!(account_object.symbol(), Some("RLUSD".to_string()));
    }
}
