use gem_encoding::protobuf::{proto_decode, proto_encode};

use super::{Command, Input};
use crate::rpc::proto::Bcs;
use crate::types;

const KIND_PROGRAMMABLE_TRANSACTION: i32 = 1;

#[derive(Clone, Debug, Default)]
pub struct Transaction {
    pub bcs: Option<Bcs>,
    pub kind: Option<TransactionKind>,
    pub sender: Option<String>,
}

impl Transaction {
    pub fn from_transaction_bcs(value: Vec<u8>) -> Self {
        Self {
            bcs: Some(Bcs::new("TransactionData", value)),
            ..Default::default()
        }
    }

    pub fn from_kind(kind: TransactionKind, sender: &str) -> Self {
        Self {
            kind: Some(kind),
            sender: Some(sender.to_string()),
            ..Default::default()
        }
    }
}

proto_encode!(Transaction {
    1 => bcs: optional_message,
    4 => kind: optional_message,
    5 => sender: optional_string,
});

proto_decode!(Transaction {
    4 => kind: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct TransactionKind {
    pub kind: Option<i32>,
    pub programmable_transaction: Option<ProgrammableTransaction>,
}

impl TransactionKind {
    pub fn programmable_transaction(transaction: ProgrammableTransaction) -> Self {
        Self {
            kind: Some(KIND_PROGRAMMABLE_TRANSACTION),
            programmable_transaction: Some(transaction),
        }
    }
}

impl From<types::TransactionKind> for TransactionKind {
    fn from(value: types::TransactionKind) -> Self {
        let types::TransactionKind::ProgrammableTransaction(transaction) = value;
        Self::programmable_transaction(transaction.into())
    }
}

proto_encode!(TransactionKind {
    1 => kind: optional_varint_i32,
    2 => programmable_transaction: optional_message,
});

proto_decode!(TransactionKind {
    2 => programmable_transaction: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct ProgrammableTransaction {
    pub inputs: Vec<Input>,
    pub commands: Vec<Command>,
}

impl From<types::ProgrammableTransaction> for ProgrammableTransaction {
    fn from(value: types::ProgrammableTransaction) -> Self {
        Self {
            inputs: value.inputs.into_iter().map(Input::from).collect(),
            commands: value.commands.into_iter().map(Command::from).collect(),
        }
    }
}

proto_encode!(ProgrammableTransaction {
    1 => inputs: repeated_message,
    2 => commands: repeated_message,
});

proto_decode!(ProgrammableTransaction {
    2 => commands: repeated_message,
});
