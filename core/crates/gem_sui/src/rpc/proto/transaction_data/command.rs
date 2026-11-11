use gem_encoding::protobuf::{encode_message_field, proto_decode, proto_encode};

use super::Argument;
use crate::types;

#[derive(Clone, Debug, Default)]
pub enum Command {
    MoveCall(MoveCall),
    TransferObjects(TransferObjects),
    SplitCoins(SplitCoins),
    MergeCoins(MergeCoins),
    Publish(Publish),
    MakeMoveVector(MakeMoveVector),
    Upgrade(Upgrade),
    #[default]
    Unknown,
}

impl From<types::Command> for Command {
    fn from(value: types::Command) -> Self {
        match value {
            types::Command::MoveCall(value) => Self::MoveCall(value.into()),
            types::Command::TransferObjects(value) => Self::TransferObjects(value.into()),
            types::Command::SplitCoins(value) => Self::SplitCoins(value.into()),
            types::Command::MergeCoins(value) => Self::MergeCoins(value.into()),
            types::Command::Publish(value) => Self::Publish(value.into()),
            types::Command::MakeMoveVector(value) => Self::MakeMoveVector(value.into()),
            types::Command::Upgrade(value) => Self::Upgrade(value.into()),
        }
    }
}

impl From<MoveCall> for Command {
    fn from(value: MoveCall) -> Self {
        Self::MoveCall(value)
    }
}

proto_encode!(Command as value {
    match value {
        Command::MoveCall(value) => encode_message_field(1, &value.encode()),
        Command::TransferObjects(value) => encode_message_field(2, &value.encode()),
        Command::SplitCoins(value) => encode_message_field(3, &value.encode()),
        Command::MergeCoins(value) => encode_message_field(4, &value.encode()),
        Command::Publish(value) => encode_message_field(5, &value.encode()),
        Command::MakeMoveVector(value) => encode_message_field(6, &value.encode()),
        Command::Upgrade(value) => encode_message_field(7, &value.encode()),
        Command::Unknown => Vec::new(),
    },
});

proto_decode!(Command {
    1 => |value, field| *value = Self::MoveCall(field.message()?),
});

#[derive(Clone, Debug, Default)]
pub struct MoveCall {
    pub package: Option<String>,
    pub module: Option<String>,
    pub function: Option<String>,
    pub type_arguments: Vec<String>,
    pub arguments: Vec<Argument>,
}

impl MoveCall {
    pub fn from_parts(package: impl ToString, module: impl Into<String>, function: impl Into<String>, arguments: Vec<Argument>) -> Self {
        Self {
            package: Some(package.to_string()),
            module: Some(module.into()),
            function: Some(function.into()),
            arguments,
            ..Default::default()
        }
    }
}

impl From<types::MoveCall> for MoveCall {
    fn from(value: types::MoveCall) -> Self {
        Self {
            package: Some(value.package.to_string()),
            module: Some(value.module.to_string()),
            function: Some(value.function.to_string()),
            type_arguments: value.type_arguments.iter().map(ToString::to_string).collect(),
            arguments: value.arguments.into_iter().map(Argument::from).collect(),
        }
    }
}

proto_encode!(MoveCall {
    1 => package: optional_string,
    2 => module: optional_string,
    3 => function: optional_string,
    4 => type_arguments: repeated_string,
    5 => arguments: repeated_message,
});

proto_decode!(MoveCall {
    1 => package: optional_string,
});

#[derive(Clone, Debug, Default)]
pub struct TransferObjects {
    pub objects: Vec<Argument>,
    pub address: Option<Argument>,
}

impl From<types::TransferObjects> for TransferObjects {
    fn from(value: types::TransferObjects) -> Self {
        Self {
            objects: value.objects.into_iter().map(Argument::from).collect(),
            address: Some(Argument::from(value.address)),
        }
    }
}

proto_encode!(TransferObjects {
    1 => objects: repeated_message,
    2 => address: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct SplitCoins {
    pub coin: Option<Argument>,
    pub amounts: Vec<Argument>,
}

impl From<types::SplitCoins> for SplitCoins {
    fn from(value: types::SplitCoins) -> Self {
        Self {
            coin: Some(Argument::from(value.coin)),
            amounts: value.amounts.into_iter().map(Argument::from).collect(),
        }
    }
}

proto_encode!(SplitCoins {
    1 => coin: optional_message,
    2 => amounts: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct MergeCoins {
    pub coin: Option<Argument>,
    pub coins_to_merge: Vec<Argument>,
}

impl From<types::MergeCoins> for MergeCoins {
    fn from(value: types::MergeCoins) -> Self {
        Self {
            coin: Some(Argument::from(value.coin)),
            coins_to_merge: value.coins_to_merge.into_iter().map(Argument::from).collect(),
        }
    }
}

proto_encode!(MergeCoins {
    1 => coin: optional_message,
    2 => coins_to_merge: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct Publish {
    pub modules: Vec<Vec<u8>>,
    pub dependencies: Vec<String>,
}

impl From<types::Publish> for Publish {
    fn from(value: types::Publish) -> Self {
        Self {
            modules: value.modules,
            dependencies: value.dependencies.iter().map(ToString::to_string).collect(),
        }
    }
}

proto_encode!(Publish {
    1 => modules: repeated_bytes,
    2 => dependencies: repeated_string,
});

#[derive(Clone, Debug, Default)]
pub struct MakeMoveVector {
    pub element_type: Option<String>,
    pub elements: Vec<Argument>,
}

impl From<types::MakeMoveVector> for MakeMoveVector {
    fn from(value: types::MakeMoveVector) -> Self {
        Self {
            element_type: value.element_type.map(|value| value.to_string()),
            elements: value.elements.into_iter().map(Argument::from).collect(),
        }
    }
}

proto_encode!(MakeMoveVector {
    1 => element_type: optional_string,
    2 => elements: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct Upgrade {
    pub modules: Vec<Vec<u8>>,
    pub dependencies: Vec<String>,
    pub package: Option<String>,
    pub ticket: Option<Argument>,
}

impl From<types::Upgrade> for Upgrade {
    fn from(value: types::Upgrade) -> Self {
        Self {
            modules: value.modules,
            dependencies: value.dependencies.iter().map(ToString::to_string).collect(),
            package: Some(value.package.to_string()),
            ticket: Some(Argument::from(value.ticket)),
        }
    }
}

proto_encode!(Upgrade {
    1 => modules: repeated_bytes,
    2 => dependencies: repeated_string,
    3 => package: optional_string,
    4 => ticket: optional_message,
});
