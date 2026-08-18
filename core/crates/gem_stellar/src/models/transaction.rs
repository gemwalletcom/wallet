use num_bigint::BigUint;
#[cfg(feature = "rpc")]
use number_formatter::BigNumberFormatter;
#[cfg(feature = "rpc")]
use serde::de;
use serde::{Deserialize, Serialize};

#[cfg(feature = "rpc")]
use crate::constants::STELLAR_DECIMALS;

#[cfg(feature = "rpc")]
fn deserialize_option_stellar_amount<'de, D>(deserializer: D) -> Result<Option<BigUint>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let amount = Option::<String>::deserialize(deserializer)?;
    amount
        .map(|amount| BigNumberFormatter::value_from_amount_biguint(&amount, STELLAR_DECIMALS).map_err(de::Error::custom))
        .transpose()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarTransactionBroadcast {
    pub hash: Option<String>,
    #[serde(rename = "title")]
    pub error_message: Option<String>,
    pub tx_status: String,
    pub error_result_xdr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarTransactionStatus {
    pub successful: bool,
    #[serde(deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub fee_charged: BigUint,
    pub hash: String,
}

// RPC models
#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub transaction_successful: bool,
    pub transaction_hash: String,
    #[serde(rename = "type")]
    pub payment_type: String,

    // payment
    pub asset_type: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "rpc", serde(deserialize_with = "deserialize_option_stellar_amount"))]
    pub amount: Option<BigUint>,

    pub created_at: String,

    // create account
    pub source_account: Option<String>,
    pub funder: Option<String>,
    pub account: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "rpc", serde(deserialize_with = "deserialize_option_stellar_amount"))]
    pub starting_balance: Option<BigUint>,

    pub transaction: Option<StellarPaymentTransaction>,
}

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarPaymentTransaction {
    pub memo: Option<String>,
}

#[cfg(feature = "rpc")]
impl Payment {
    pub fn from_address(&self) -> Option<String> {
        use crate::constants::{TRANSACTION_TYPE_CREATE_ACCOUNT, TRANSACTION_TYPE_PAYMENT};
        match self.payment_type.as_str() {
            TRANSACTION_TYPE_PAYMENT => self.from.clone(),
            TRANSACTION_TYPE_CREATE_ACCOUNT => self.funder.clone(),
            _ => None,
        }
    }

    pub fn to_address(&self) -> Option<String> {
        use crate::constants::{TRANSACTION_TYPE_CREATE_ACCOUNT, TRANSACTION_TYPE_PAYMENT};
        match self.payment_type.as_str() {
            TRANSACTION_TYPE_PAYMENT => self.to.clone(),
            TRANSACTION_TYPE_CREATE_ACCOUNT => self.account.clone(),
            _ => None,
        }
    }

    pub fn get_state(&self) -> primitives::TransactionState {
        use primitives::TransactionState;
        match self.transaction_successful {
            true => TransactionState::Confirmed,
            false => TransactionState::Failed,
        }
    }

    pub fn get_value(&self) -> Option<String> {
        use crate::constants::{TRANSACTION_TYPE_CREATE_ACCOUNT, TRANSACTION_TYPE_PAYMENT};
        match self.payment_type.as_str() {
            TRANSACTION_TYPE_PAYMENT => Some(self.amount.as_ref()?.to_string()),
            TRANSACTION_TYPE_CREATE_ACCOUNT => Some(self.starting_balance.as_ref()?.to_string()),
            _ => None,
        }
    }

    pub fn get_memo(&self) -> Option<String> {
        self.transaction.as_ref()?.memo.clone()
    }
}
