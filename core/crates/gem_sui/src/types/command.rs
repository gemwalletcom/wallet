use serde::{Deserialize, Serialize};

use super::{Address, Identifier, TypeTag};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    MoveCall(MoveCall),
    TransferObjects(TransferObjects),
    SplitCoins(SplitCoins),
    MergeCoins(MergeCoins),
    Publish(Publish),
    MakeMoveVector(MakeMoveVector),
    Upgrade(Upgrade),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Argument {
    Gas,
    Input(u16),
    Result(u16),
    NestedResult(u16, u16),
}

impl Argument {
    pub fn nested(self, index: u16) -> Option<Self> {
        match self {
            Self::Result(command) => Some(Self::NestedResult(command, index)),
            Self::Gas | Self::Input(_) | Self::NestedResult(_, _) => None,
        }
    }

    pub fn to_nested(self, count: u16) -> Vec<Self> {
        (0..count).filter_map(|index| self.nested(index)).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveCall {
    pub package: Address,
    pub module: Identifier,
    pub function: Identifier,
    pub type_arguments: Vec<TypeTag>,
    pub arguments: Vec<Argument>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferObjects {
    pub objects: Vec<Argument>,
    pub address: Argument,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitCoins {
    pub coin: Argument,
    pub amounts: Vec<Argument>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeCoins {
    pub coin: Argument,
    pub coins_to_merge: Vec<Argument>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Publish {
    pub modules: Vec<Vec<u8>>,
    pub dependencies: Vec<Address>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MakeMoveVector {
    pub element_type: Option<TypeTag>,
    pub elements: Vec<Argument>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Upgrade {
    pub modules: Vec<Vec<u8>>,
    pub dependencies: Vec<Address>,
    pub package: Address,
    pub ticket: Argument,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_to_nested() {
        assert_eq!(Argument::Result(3).to_nested(2), vec![Argument::NestedResult(3, 0), Argument::NestedResult(3, 1)]);
        assert_eq!(Argument::Gas.to_nested(2), vec![]);
        assert_eq!(Argument::Input(3).to_nested(2), vec![]);
        assert_eq!(Argument::NestedResult(3, 0).to_nested(2), vec![]);
    }
}
