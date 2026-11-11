use serde::{Deserialize, Serialize};

use super::{Address, Digest, TypeTag};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Input {
    Pure(Vec<u8>),
    Object(ObjectInput),
    FundsWithdrawal(FundsWithdrawal),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectInput {
    ImmutableOrOwned(ObjectReference),
    Shared(SharedObject),
    Receiving(ObjectReference),
}

impl ObjectInput {
    pub fn immutable_or_owned(object_id: Address, version: u64, digest: Digest) -> Self {
        Self::ImmutableOrOwned(ObjectReference { object_id, version, digest })
    }

    pub fn receiving(object_id: Address, version: u64, digest: Digest) -> Self {
        Self::Receiving(ObjectReference { object_id, version, digest })
    }

    pub fn shared(object_id: Address, initial_shared_version: u64, mutable: bool) -> Self {
        let mutability = if mutable { Mutability::Mutable } else { Mutability::Immutable };
        Self::Shared(SharedObject {
            object_id,
            initial_shared_version,
            mutability,
        })
    }

    pub fn object_id(&self) -> Address {
        match self {
            Self::ImmutableOrOwned(reference) | Self::Receiving(reference) => reference.object_id,
            Self::Shared(shared) => shared.object_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectReference {
    pub object_id: Address,
    pub version: u64,
    pub digest: Digest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedObject {
    pub object_id: Address,
    pub initial_shared_version: u64,
    pub mutability: Mutability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mutability {
    Immutable,
    Mutable,
    NonExclusiveWrite,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FundsWithdrawal {
    pub reservation: WithdrawalReservation,
    pub type_arg: WithdrawalType,
    pub withdraw_from: WithdrawFrom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WithdrawalReservation {
    MaxAmount(u64),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WithdrawalType {
    Balance(TypeTag),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WithdrawFrom {
    Sender,
    Sponsor,
    SenderAllowance { funder: Address, allowance: Address },
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::hex;

    #[test]
    fn test_funds_withdrawal_sender_allowance_bcs() {
        let input = Input::FundsWithdrawal(FundsWithdrawal {
            reservation: WithdrawalReservation::MaxAmount(7),
            type_arg: WithdrawalType::Balance("0x2::sui::SUI".parse().unwrap()),
            withdraw_from: WithdrawFrom::SenderAllowance {
                funder: Address::new([1; 32]),
                allowance: Address::new([2; 32]),
            },
        });
        let bytes = bcs::to_bytes(&input).unwrap();

        assert_eq!(
            hex::encode(&bytes),
            "02000700000000000000000700000000000000000000000000000000000000000000000000000000000000020373756903535549000201010101010101010101010101010101010101010101010101010101010101010202020202020202020202020202020202020202020202020202020202020202"
        );
        assert_eq!(bcs::from_bytes::<Input>(&bytes).unwrap(), input);
    }
}
