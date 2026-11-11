use serde::{Deserialize, Serialize};

use super::{Address, Command, Digest, Input, ObjectReference};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "super::serializer::TransactionData", into = "super::serializer::TransactionData")]
pub struct Transaction {
    pub kind: TransactionKind,
    pub sender: Address,
    pub gas_payment: GasPayment,
    pub expiration: TransactionExpiration,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionKind {
    ProgrammableTransaction(ProgrammableTransaction),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgrammableTransaction {
    pub inputs: Vec<Input>,
    pub commands: Vec<Command>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GasPayment {
    pub objects: Vec<ObjectReference>,
    pub owner: Address,
    pub price: u64,
    pub budget: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionExpiration {
    None,
    Epoch(u64),
    ValidDuring {
        min_epoch: Option<u64>,
        max_epoch: Option<u64>,
        min_timestamp: Option<u64>,
        max_timestamp: Option<u64>,
        chain: Digest,
        nonce: u32,
    },
    Validity {
        min_epoch: Option<u64>,
        max_epoch: Option<u64>,
        min_timestamp: Option<u64>,
        max_timestamp: Option<u64>,
        chain: Digest,
        nonce: u32,
        allowed_proposers: Option<AllowedProposers>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowedProposers {
    pub epoch: u64,
    #[serde(deserialize_with = "super::serializer::deserialize_proposers")]
    pub proposers: Vec<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_proposers_bcs() {
        let proposers = AllowedProposers { epoch: 1, proposers: vec![3] };
        let bytes = bcs::to_bytes(&proposers).unwrap();

        assert_eq!(bcs::from_bytes::<AllowedProposers>(&bytes).unwrap(), proposers);
        assert!(bcs::from_bytes::<AllowedProposers>(&bytes[..9]).is_err());
        assert!(bcs::from_bytes::<AllowedProposers>(&[1, 0, 0, 0, 0, 0, 0, 0, 0]).is_err());
    }
}
