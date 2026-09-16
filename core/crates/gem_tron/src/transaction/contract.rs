use primitives::{Resource, SignerError};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use serde_serializers::hex_bytes;

use crate::{address::TronAddress, models::TronContractType};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
#[repr(u64)]
pub(crate) enum TronResource {
    #[default]
    Bandwidth = 0,
    Energy = 1,
}

impl From<&Resource> for TronResource {
    fn from(resource: &Resource) -> Self {
        match resource {
            Resource::Bandwidth => Self::Bandwidth,
            Resource::Energy => Self::Energy,
        }
    }
}

impl From<TronResource> for u64 {
    fn from(resource: TronResource) -> Self {
        resource as u64
    }
}

#[derive(Debug)]
pub(crate) struct TronContractVote {
    pub(crate) address: TronAddress,
    pub(crate) count: u64,
}

#[derive(Debug)]
pub(crate) enum TronContract {
    Transfer {
        owner: TronAddress,
        to: TronAddress,
        amount: u64,
    },
    TriggerSmart {
        owner: TronAddress,
        contract: TronAddress,
        data: Vec<u8>,
        call_value: Option<u64>,
        call_token_value: Option<u64>,
        token_id: Option<u64>,
    },
    VoteWitness {
        owner: TronAddress,
        votes: Vec<TronContractVote>,
        support: bool,
    },
    FreezeBalanceV2 {
        owner: TronAddress,
        frozen_balance: u64,
        resource: TronResource,
    },
    UnfreezeBalanceV2 {
        owner: TronAddress,
        unfreeze_balance: u64,
        resource: TronResource,
    },
    DelegateResource {
        owner: TronAddress,
        receiver: TronAddress,
        balance: u64,
        resource: TronResource,
        lock: bool,
        lock_period: u64,
    },
    UnDelegateResource {
        owner: TronAddress,
        receiver: TronAddress,
        balance: u64,
        resource: TronResource,
    },
    WithdrawBalance {
        owner: TronAddress,
    },
    WithdrawExpireUnfreeze {
        owner: TronAddress,
    },
}

impl TronContract {
    pub(crate) fn kind(&self) -> TronContractType {
        match self {
            Self::Transfer { .. } => TronContractType::Transfer,
            Self::TriggerSmart { .. } => TronContractType::TriggerSmart,
            Self::VoteWitness { .. } => TronContractType::VoteWitness,
            Self::FreezeBalanceV2 { .. } => TronContractType::FreezeBalanceV2,
            Self::UnfreezeBalanceV2 { .. } => TronContractType::UnfreezeBalanceV2,
            Self::DelegateResource { .. } => TronContractType::DelegateResource,
            Self::UnDelegateResource { .. } => TronContractType::UnDelegateResource,
            Self::WithdrawBalance { .. } => TronContractType::WithdrawBalance,
            Self::WithdrawExpireUnfreeze { .. } => TronContractType::WithdrawExpireUnfreeze,
        }
    }

    pub(crate) fn json(&self) -> TronContractJson {
        let contract_type = self.kind();
        TronContractJson {
            parameter: TronContractParameterJson {
                type_url: contract_type.type_url(),
                value: self.value_json(),
            },
            contract_type,
        }
    }

    fn value_json(&self) -> TronContractValueJson {
        match self {
            Self::Transfer { owner, to, amount } => TronContractValueJson::Transfer(TransferContractValue {
                amount: *amount,
                owner_address: *owner,
                to_address: *to,
            }),
            Self::TriggerSmart {
                owner,
                contract,
                data,
                call_value,
                call_token_value,
                token_id,
            } => TronContractValueJson::TriggerSmart(TriggerSmartContractValue {
                contract_address: *contract,
                data: data.clone(),
                owner_address: *owner,
                call_value: call_value.filter(|value| *value > 0),
                call_token_value: call_token_value.filter(|value| *value > 0),
                token_id: token_id.filter(|value| *value > 0),
            }),
            Self::VoteWitness { owner, votes, support } => TronContractValueJson::VoteWitness(VoteWitnessContractValue {
                owner_address: *owner,
                support: *support,
                votes: votes.iter().map(VoteValue::from).collect(),
            }),
            Self::FreezeBalanceV2 { owner, frozen_balance, resource } => TronContractValueJson::FreezeBalanceV2(FreezeBalanceV2ContractValue {
                frozen_balance: *frozen_balance,
                owner_address: *owner,
                resource: *resource,
            }),
            Self::UnfreezeBalanceV2 {
                owner,
                unfreeze_balance,
                resource,
            } => TronContractValueJson::UnfreezeBalanceV2(UnfreezeBalanceV2ContractValue {
                owner_address: *owner,
                resource: *resource,
                unfreeze_balance: *unfreeze_balance,
            }),
            Self::DelegateResource {
                owner,
                receiver,
                balance,
                resource,
                lock,
                lock_period,
            } => TronContractValueJson::DelegateResource(DelegateResourceContractValue {
                owner_address: *owner,
                receiver_address: *receiver,
                balance: *balance,
                resource: *resource,
                lock: *lock,
                lock_period: *lock_period,
            }),
            Self::UnDelegateResource {
                owner,
                receiver,
                balance,
                resource,
            } => TronContractValueJson::UnDelegateResource(UnDelegateResourceContractValue {
                owner_address: *owner,
                receiver_address: *receiver,
                balance: *balance,
                resource: *resource,
            }),
            Self::WithdrawBalance { owner } | Self::WithdrawExpireUnfreeze { owner } => TronContractValueJson::Owner(OwnerContractValue { owner_address: *owner }),
        }
    }

    pub(crate) fn from_json_value(contract_type: TronContractType, value: Value) -> Result<Self, SignerError> {
        match contract_type {
            TronContractType::Transfer => {
                let value: TransferContractValue = serde_json::from_value(value)?;
                Ok(Self::Transfer {
                    owner: value.owner_address,
                    to: value.to_address,
                    amount: value.amount,
                })
            }
            TronContractType::TriggerSmart => {
                let value: TriggerSmartContractValue = serde_json::from_value(value)?;
                Ok(Self::TriggerSmart {
                    owner: value.owner_address,
                    contract: value.contract_address,
                    data: value.data,
                    call_value: value.call_value.filter(|value| *value > 0),
                    call_token_value: value.call_token_value.filter(|value| *value > 0),
                    token_id: value.token_id.filter(|value| *value > 0),
                })
            }
            TronContractType::VoteWitness => {
                let value: VoteWitnessContractValue = serde_json::from_value(value)?;
                Ok(Self::VoteWitness {
                    owner: value.owner_address,
                    votes: value.votes.into_iter().map(TronContractVote::from).collect(),
                    support: value.support,
                })
            }
            TronContractType::FreezeBalanceV2 => {
                let value: FreezeBalanceV2ContractValue = serde_json::from_value(value)?;
                Ok(Self::FreezeBalanceV2 {
                    owner: value.owner_address,
                    frozen_balance: value.frozen_balance,
                    resource: value.resource,
                })
            }
            TronContractType::UnfreezeBalanceV2 => {
                let value: UnfreezeBalanceV2ContractValue = serde_json::from_value(value)?;
                Ok(Self::UnfreezeBalanceV2 {
                    owner: value.owner_address,
                    unfreeze_balance: value.unfreeze_balance,
                    resource: value.resource,
                })
            }
            TronContractType::WithdrawBalance => {
                let value: OwnerContractValue = serde_json::from_value(value)?;
                Ok(Self::WithdrawBalance { owner: value.owner_address })
            }
            TronContractType::WithdrawExpireUnfreeze => {
                let value: OwnerContractValue = serde_json::from_value(value)?;
                Ok(Self::WithdrawExpireUnfreeze { owner: value.owner_address })
            }
            TronContractType::DelegateResource => {
                let value: DelegateResourceContractValue = serde_json::from_value(value)?;
                Ok(Self::DelegateResource {
                    owner: value.owner_address,
                    receiver: value.receiver_address,
                    balance: value.balance,
                    resource: value.resource,
                    lock: value.lock,
                    lock_period: value.lock_period,
                })
            }
            TronContractType::UnDelegateResource => {
                let value: UnDelegateResourceContractValue = serde_json::from_value(value)?;
                Ok(Self::UnDelegateResource {
                    owner: value.owner_address,
                    receiver: value.receiver_address,
                    balance: value.balance,
                    resource: value.resource,
                })
            }
            TronContractType::TransferAsset => Err(SignerError::invalid_input(format!("unsupported Tron contract type: {contract_type}"))),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct TronContractJson {
    parameter: TronContractParameterJson,
    #[serde(rename = "type")]
    contract_type: TronContractType,
}

#[derive(Debug, Serialize)]
struct TronContractParameterJson {
    type_url: String,
    value: TronContractValueJson,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum TronContractValueJson {
    Transfer(TransferContractValue),
    TriggerSmart(TriggerSmartContractValue),
    VoteWitness(VoteWitnessContractValue),
    FreezeBalanceV2(FreezeBalanceV2ContractValue),
    UnfreezeBalanceV2(UnfreezeBalanceV2ContractValue),
    DelegateResource(DelegateResourceContractValue),
    UnDelegateResource(UnDelegateResourceContractValue),
    Owner(OwnerContractValue),
}

#[derive(Debug, Deserialize, Serialize)]
struct TransferContractValue {
    amount: u64,
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    owner_address: TronAddress,
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    to_address: TronAddress,
}

#[derive(Debug, Deserialize, Serialize)]
struct TriggerSmartContractValue {
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    contract_address: TronAddress,
    #[serde(default, with = "hex_bytes")]
    data: Vec<u8>,
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    owner_address: TronAddress,
    #[serde(skip_serializing_if = "Option::is_none")]
    call_value: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    call_token_value: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_id: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct VoteWitnessContractValue {
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    owner_address: TronAddress,
    #[serde(default)]
    support: bool,
    #[serde(default)]
    votes: Vec<VoteValue>,
}

#[derive(Debug, Deserialize, Serialize)]
struct VoteValue {
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    vote_address: TronAddress,
    vote_count: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct FreezeBalanceV2ContractValue {
    frozen_balance: u64,
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    owner_address: TronAddress,
    #[serde(default)]
    resource: TronResource,
}

#[derive(Debug, Deserialize, Serialize)]
struct UnfreezeBalanceV2ContractValue {
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    owner_address: TronAddress,
    #[serde(default)]
    resource: TronResource,
    unfreeze_balance: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct DelegateResourceContractValue {
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    owner_address: TronAddress,
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    receiver_address: TronAddress,
    balance: u64,
    #[serde(default)]
    resource: TronResource,
    #[serde(default)]
    lock: bool,
    #[serde(default)]
    lock_period: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct UnDelegateResourceContractValue {
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    owner_address: TronAddress,
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    receiver_address: TronAddress,
    balance: u64,
    #[serde(default)]
    resource: TronResource,
}

#[derive(Debug, Deserialize, Serialize)]
struct OwnerContractValue {
    #[serde(with = "crate::address::serializer::hex_or_base58")]
    owner_address: TronAddress,
}

impl From<&TronContractVote> for VoteValue {
    fn from(vote: &TronContractVote) -> Self {
        Self {
            vote_address: vote.address,
            vote_count: vote.count,
        }
    }
}

impl From<VoteValue> for TronContractVote {
    fn from(vote: VoteValue) -> Self {
        Self {
            address: vote.vote_address,
            count: vote.vote_count,
        }
    }
}

impl Serialize for TronContract {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.json().serialize(serializer)
    }
}
