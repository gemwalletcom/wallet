use gem_encoding::protobuf::{MessageEncode, proto_encode};
use primitives::Address as _;

use super::{TronContract, TronContractVote};

#[derive(Clone, Debug, Default)]
pub(crate) struct RawData {
    pub(crate) ref_block_bytes: Option<Vec<u8>>,
    pub(crate) ref_block_hash: Option<Vec<u8>>,
    pub(crate) expiration: Option<u64>,
    pub(crate) data: Option<Vec<u8>>,
    pub(crate) contracts: Vec<ContractEnvelope>,
    pub(crate) timestamp: Option<u64>,
    pub(crate) fee_limit: Option<u64>,
}

proto_encode!(RawData {
    1 => ref_block_bytes: optional_bytes,
    4 => ref_block_hash: optional_bytes,
    8 => expiration: optional_varint_u64,
    10 => data: optional_bytes,
    11 => contracts: repeated_message,
    14 => timestamp: optional_varint_u64,
    18 => fee_limit: optional_varint_u64,
});

#[derive(Clone, Debug, Default)]
pub(crate) struct BlockHeaderRaw {
    pub(crate) timestamp: Option<u64>,
    pub(crate) tx_trie_root: Option<Vec<u8>>,
    pub(crate) parent_hash: Option<Vec<u8>>,
    pub(crate) number: Option<u64>,
    pub(crate) witness_address: Option<Vec<u8>>,
    pub(crate) version: Option<u64>,
}

proto_encode!(BlockHeaderRaw {
    1 => timestamp: optional_varint_u64,
    2 => tx_trie_root: optional_bytes,
    3 => parent_hash: optional_bytes,
    7 => number: optional_varint_u64,
    9 => witness_address: optional_bytes,
    10 => version: optional_varint_u64,
});

#[derive(Clone, Debug, Default)]
pub(crate) struct ContractEnvelope {
    contract_type: Option<u64>,
    parameter: Option<AnyParameter>,
}

proto_encode!(ContractEnvelope {
    1 => contract_type: optional_varint_u64,
    2 => parameter: optional_message,
});

impl From<&TronContract> for ContractEnvelope {
    fn from(contract: &TronContract) -> Self {
        let contract_type = contract.kind();
        Self {
            contract_type: Some(contract_type.id()),
            parameter: Some(AnyParameter {
                type_url: Some(contract_type.type_url()),
                value: Some(contract_value(contract)),
            }),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct AnyParameter {
    type_url: Option<String>,
    value: Option<Vec<u8>>,
}

proto_encode!(AnyParameter {
    1 => type_url: optional_string,
    2 => value: optional_bytes,
});

#[derive(Clone, Debug, Default)]
struct TransferContract {
    owner_address: Option<Vec<u8>>,
    to_address: Option<Vec<u8>>,
    amount: Option<u64>,
}

proto_encode!(TransferContract {
    1 => owner_address: optional_bytes,
    2 => to_address: optional_bytes,
    3 => amount: optional_varint_u64,
});

#[derive(Clone, Debug, Default)]
struct TriggerSmartContract {
    owner_address: Option<Vec<u8>>,
    contract_address: Option<Vec<u8>>,
    call_value: Option<u64>,
    data: Option<Vec<u8>>,
    call_token_value: Option<u64>,
    token_id: Option<u64>,
}

proto_encode!(TriggerSmartContract {
    1 => owner_address: optional_bytes,
    2 => contract_address: optional_bytes,
    3 => call_value: optional_varint_u64,
    4 => data: optional_bytes,
    5 => call_token_value: optional_varint_u64,
    6 => token_id: optional_varint_u64,
});

#[derive(Clone, Debug, Default)]
struct VoteWitnessContract {
    owner_address: Option<Vec<u8>>,
    votes: Vec<Vote>,
    support: Option<bool>,
}

proto_encode!(VoteWitnessContract {
    1 => owner_address: optional_bytes,
    2 => votes: repeated_message,
    3 => support: optional_bool,
});

#[derive(Clone, Debug, Default)]
struct Vote {
    vote_address: Option<Vec<u8>>,
    vote_count: Option<u64>,
}

proto_encode!(Vote {
    1 => vote_address: optional_bytes,
    2 => vote_count: optional_varint_u64,
});

impl From<&TronContractVote> for Vote {
    fn from(vote: &TronContractVote) -> Self {
        Self {
            vote_address: Some(vote.address.as_bytes().to_vec()),
            vote_count: (vote.count > 0).then_some(vote.count),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct FreezeBalanceV2Contract {
    owner_address: Option<Vec<u8>>,
    frozen_balance: Option<u64>,
    resource: Option<u64>,
}

proto_encode!(FreezeBalanceV2Contract {
    1 => owner_address: optional_bytes,
    2 => frozen_balance: optional_varint_u64,
    3 => resource: optional_varint_u64,
});

#[derive(Clone, Debug, Default)]
struct UnfreezeBalanceV2Contract {
    owner_address: Option<Vec<u8>>,
    unfreeze_balance: Option<u64>,
    resource: Option<u64>,
}

proto_encode!(UnfreezeBalanceV2Contract {
    1 => owner_address: optional_bytes,
    2 => unfreeze_balance: optional_varint_u64,
    3 => resource: optional_varint_u64,
});

#[derive(Clone, Debug, Default)]
struct OwnerContract {
    owner_address: Option<Vec<u8>>,
}

proto_encode!(OwnerContract {
    1 => owner_address: optional_bytes,
});

fn contract_value(contract: &TronContract) -> Vec<u8> {
    match contract {
        TronContract::Transfer { owner, to, amount } => TransferContract {
            owner_address: Some(owner.as_bytes().to_vec()),
            to_address: Some(to.as_bytes().to_vec()),
            amount: (*amount > 0).then_some(*amount),
        }
        .encode(),
        TronContract::TriggerSmart {
            owner,
            contract,
            data,
            call_value,
            call_token_value,
            token_id,
        } => TriggerSmartContract {
            owner_address: Some(owner.as_bytes().to_vec()),
            contract_address: Some(contract.as_bytes().to_vec()),
            call_value: call_value.filter(|value| *value > 0),
            data: Some(data.clone()),
            call_token_value: call_token_value.filter(|value| *value > 0),
            token_id: token_id.filter(|value| *value > 0),
        }
        .encode(),
        TronContract::VoteWitness { owner, votes, support } => VoteWitnessContract {
            owner_address: Some(owner.as_bytes().to_vec()),
            votes: votes.iter().map(Vote::from).collect(),
            support: support.then_some(true),
        }
        .encode(),
        TronContract::FreezeBalanceV2 { owner, frozen_balance, resource } => FreezeBalanceV2Contract {
            owner_address: Some(owner.as_bytes().to_vec()),
            frozen_balance: (*frozen_balance > 0).then_some(*frozen_balance),
            resource: {
                let resource = u64::from(*resource);
                (resource > 0).then_some(resource)
            },
        }
        .encode(),
        TronContract::UnfreezeBalanceV2 {
            owner,
            unfreeze_balance,
            resource,
        } => UnfreezeBalanceV2Contract {
            owner_address: Some(owner.as_bytes().to_vec()),
            unfreeze_balance: (*unfreeze_balance > 0).then_some(*unfreeze_balance),
            resource: {
                let resource = u64::from(*resource);
                (resource > 0).then_some(resource)
            },
        }
        .encode(),
        TronContract::WithdrawBalance { owner } | TronContract::WithdrawExpireUnfreeze { owner } => OwnerContract {
            owner_address: Some(owner.as_bytes().to_vec()),
        }
        .encode(),
    }
}
