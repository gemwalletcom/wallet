use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_serializers::{deserialize_option_u64_from_str_or_int, deserialize_u64_from_str_or_int};

use crate::{SUI_COIN_TYPE, coin_type_matches};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionBuilderJson {
    pub version: u8,
    pub sender: Option<String>,
    pub expiration: Option<Value>,
    pub gas_data: Option<GasData>,
    pub inputs: Vec<TransactionInput>,
    pub commands: Vec<TransactionCommand>,
}

impl TransactionBuilderJson {
    pub fn uses_gas_coin(&self) -> bool {
        self.commands.iter().flat_map(TransactionCommand::arguments).any(TransactionArgument::is_gas_coin)
    }

    pub fn withdraws_sui(&self) -> bool {
        self.inputs.iter().any(TransactionInput::withdraws_sui)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GasData {
    #[serde(default, deserialize_with = "deserialize_option_u64_from_str_or_int")]
    pub budget: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_option_u64_from_str_or_int")]
    pub price: Option<u64>,
    pub owner: Option<String>,
    pub payment: Option<Vec<ObjectRef>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum TransactionInput {
    Pure {
        #[serde(rename = "Pure")]
        pure: PureInput,
    },
    Object {
        #[serde(rename = "Object")]
        object: TransactionObject,
    },
    UnresolvedObject {
        #[serde(rename = "UnresolvedObject")]
        object: UnresolvedObject,
    },
    FundsWithdrawal {
        #[serde(rename = "FundsWithdrawal")]
        funds_withdrawal: FundsWithdrawalInput,
    },
    UnresolvedPure {
        #[serde(rename = "UnresolvedPure")]
        pure: Value,
    },
}

impl TransactionInput {
    pub fn withdraws_sui(&self) -> bool {
        match self {
            Self::FundsWithdrawal { funds_withdrawal } => match &funds_withdrawal.type_arg {
                WithdrawalTypeArg::Balance { balance } => coin_type_matches(balance, SUI_COIN_TYPE),
            },
            Self::Pure { .. } | Self::Object { .. } | Self::UnresolvedObject { .. } | Self::UnresolvedPure { .. } => false,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundsWithdrawalInput {
    pub reservation: WithdrawalReservation,
    pub type_arg: WithdrawalTypeArg,
    pub withdraw_from: WithdrawalSource,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum WithdrawalReservation {
    MaxAmountU64 {
        #[serde(rename = "MaxAmountU64", deserialize_with = "deserialize_u64_from_str_or_int")]
        max_amount: u64,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum WithdrawalTypeArg {
    Balance {
        #[serde(rename = "Balance")]
        balance: String,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum WithdrawalSource {
    Sender {
        #[serde(rename = "Sender")]
        sender: bool,
    },
    Sponsor {
        #[serde(rename = "Sponsor")]
        sponsor: bool,
    },
}

#[derive(Clone, Debug, Deserialize)]
pub struct PureInput {
    pub bytes: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnresolvedObject {
    pub object_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum TransactionObject {
    ImmOrOwnedObject {
        #[serde(rename = "ImmOrOwnedObject")]
        object: ObjectRef,
    },
    SharedObject {
        #[serde(rename = "SharedObject")]
        object: SharedObjectRef,
    },
    Receiving {
        #[serde(rename = "Receiving")]
        object: ObjectRef,
    },
}

impl TransactionObject {
    pub fn object_id(&self) -> &str {
        match self {
            Self::ImmOrOwnedObject { object } | Self::Receiving { object } => &object.object_id,
            Self::SharedObject { object } => &object.object_id,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectRef {
    pub object_id: String,
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub version: u64,
    pub digest: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedObjectRef {
    pub object_id: String,
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub initial_shared_version: u64,
    pub mutable: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum TransactionCommand {
    MoveCall {
        #[serde(rename = "MoveCall")]
        move_call: MoveCallCommand,
    },
    TransferObjects {
        #[serde(rename = "TransferObjects")]
        transfer_objects: TransferObjectsCommand,
    },
    SplitCoins {
        #[serde(rename = "SplitCoins")]
        split_coins: SplitCoinsCommand,
    },
    MergeCoins {
        #[serde(rename = "MergeCoins")]
        merge_coins: MergeCoinsCommand,
    },
    MakeMoveVec {
        #[serde(rename = "MakeMoveVec")]
        make_move_vec: MakeMoveVecCommand,
    },
    Publish {
        #[serde(rename = "Publish")]
        publish: Value,
    },
    Upgrade {
        #[serde(rename = "Upgrade")]
        upgrade: Value,
    },
}

impl TransactionCommand {
    pub fn arguments(&self) -> Vec<&TransactionArgument> {
        match self {
            Self::MoveCall { move_call } => move_call.arguments.iter().collect(),
            Self::TransferObjects { transfer_objects } => transfer_objects.objects.iter().chain([&transfer_objects.address]).collect(),
            Self::SplitCoins { split_coins } => [&split_coins.coin].into_iter().chain(&split_coins.amounts).collect(),
            Self::MergeCoins { merge_coins } => [&merge_coins.destination].into_iter().chain(&merge_coins.sources).collect(),
            Self::MakeMoveVec { make_move_vec } => make_move_vec.elements.iter().collect(),
            Self::Publish { .. } | Self::Upgrade { .. } => Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveCallCommand {
    pub package: String,
    pub module: String,
    pub function: String,
    #[serde(default)]
    pub type_arguments: Vec<String>,
    #[serde(default)]
    pub arguments: Vec<TransactionArgument>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TransferObjectsCommand {
    pub objects: Vec<TransactionArgument>,
    pub address: TransactionArgument,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SplitCoinsCommand {
    pub coin: TransactionArgument,
    pub amounts: Vec<TransactionArgument>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MergeCoinsCommand {
    pub destination: TransactionArgument,
    pub sources: Vec<TransactionArgument>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MakeMoveVecCommand {
    pub r#type: Option<String>,
    pub elements: Vec<TransactionArgument>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum TransactionArgument {
    GasCoin {
        #[serde(rename = "GasCoin")]
        gas_coin: bool,
    },
    Input {
        #[serde(rename = "Input")]
        input: usize,
    },
    Result {
        #[serde(rename = "Result")]
        result: usize,
    },
    NestedResult {
        #[serde(rename = "NestedResult")]
        nested_result: [usize; 2],
    },
}

impl TransactionArgument {
    pub fn is_gas_coin(&self) -> bool {
        match self {
            Self::GasCoin { .. } => true,
            Self::Input { .. } | Self::Result { .. } | Self::NestedResult { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_transaction_json() {
        let transaction: TransactionBuilderJson = serde_json::from_str(include_str!("../../../testdata/transaction_builder_json.json")).unwrap();

        assert_eq!(transaction.version, 2);
        assert_eq!(transaction.inputs.len(), 2);
        assert_eq!(transaction.commands.len(), 2);
        assert_eq!(transaction.sender, None);
        assert!(transaction.gas_data.is_none());
    }

    #[test]
    fn test_decode_wallet_connect_sign_transaction_json() {
        let transaction: TransactionBuilderJson = serde_json::from_str(include_str!("../../../testdata/wc_sign_transaction_cetus.json")).unwrap();

        assert_eq!(transaction.inputs.len(), 23);
        assert_eq!(transaction.commands.len(), 8);
        assert_eq!(transaction.gas_data.unwrap().budget, Some(50_000_000));

        match &transaction.inputs[20] {
            TransactionInput::Object {
                object: TransactionObject::ImmOrOwnedObject { object },
            } => assert_eq!(object.version, 936480130),
            other => panic!("expected owned object input, got {other:?}"),
        }
        match &transaction.commands[2] {
            TransactionCommand::MoveCall { move_call } => {
                assert_eq!(move_call.arguments[3], TransactionArgument::NestedResult { nested_result: [1, 0] });
            }
            other => panic!("expected move call command, got {other:?}"),
        }
    }
}
