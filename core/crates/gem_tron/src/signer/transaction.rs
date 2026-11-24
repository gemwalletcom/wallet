use gem_hash::sha2::sha256;
use primitives::hex;
use primitives::{SignerError, SignerInput, StakeType, TransactionLoadMetadata, TronStakeData, decode_hex, swap::SwapQuoteDataType};
use signer::{SignatureScheme, Signer};

use super::builder::{SignedTransactionJson, TronRawData};
use super::wallet_connect::WalletConnectPayload;
use crate::address::TronAddress;
use crate::transaction::{TronContract, TronTriggerSmartContract, contract::TronResource};

struct ContractPayload {
    contract: TronContract,
    data: Option<Vec<u8>>,
}

impl From<TronContract> for ContractPayload {
    fn from(contract: TronContract) -> Self {
        Self { contract, data: None }
    }
}

pub(crate) fn sign_transfer(input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
    sign_contract_payload(input, private_key, |owner| {
        Ok(TronContract::Transfer {
            owner,
            to: TronAddress::parse(&input.destination_address)?,
            amount: input.value_as_u64()?,
        }
        .into())
    })
}

pub(crate) fn sign_token_transfer(input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
    sign_contract_payload(input, private_key, |owner| {
        let call = TronTriggerSmartContract::token_transfer(owner, input.input_type.get_asset().id.get_token_id()?, &input.destination_address, &input.value)?;
        Ok(TronContract::TriggerSmart(call).into())
    })
}

pub(crate) fn sign_token_approval(input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
    sign_contract_payload(input, private_key, |owner| {
        Ok(TronContract::TriggerSmart(TronTriggerSmartContract::token_approval(owner, input.input_type.get_approval_data()?)?).into())
    })
}

pub(crate) fn sign_swap(input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
    let swap_data = &input.input_type.get_swap_data()?.data;

    match swap_data.data_type {
        SwapQuoteDataType::Transfer => SignerError::invalid_input_err("unsupported Tron transfer swap"),
        SwapQuoteDataType::Contract => sign_contract_payloads(input, private_key, |owner| {
            let swap_payload = ContractPayload {
                contract: TronContract::contract_swap(owner, input.input_type.get_asset(), swap_data)?,
                data: swap_data.memo.as_deref().map(decode_hex).transpose()?,
            };
            match &swap_data.approval {
                Some(approval) => Ok(vec![TronContract::TriggerSmart(TronTriggerSmartContract::token_approval(owner, approval)?).into(), swap_payload]),
                None => Ok(vec![swap_payload]),
            }
        }),
    }
}

pub(crate) fn sign_stake(input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
    let stake_type = input.input_type.get_stake_type().map_err(SignerError::invalid_input)?;
    let TransactionLoadMetadata::Tron { stake_data, .. } = &input.metadata else {
        return SignerError::invalid_input_err("Missing tron metadata");
    };

    sign_contract_payloads(input, private_key, |owner| {
        let contracts = match stake_type {
            StakeType::Stake(_) | StakeType::Redelegate(_) => match stake_data {
                TronStakeData::Votes { votes } => vec![TronContract::vote_witness(owner, votes)?],
                TronStakeData::Unfreeze { unfreezes: _ } => return SignerError::invalid_input_err("Expected Tron vote stake data"),
            },
            StakeType::Unstake(_) => match stake_data {
                TronStakeData::Votes { votes } => vec![TronContract::vote_witness(owner, votes)?],
                TronStakeData::Unfreeze { unfreezes } => unfreezes
                    .iter()
                    .map(|unfreeze| TronContract::UnfreezeBalanceV2 {
                        owner,
                        unfreeze_balance: unfreeze.amount,
                        resource: TronResource::from(&unfreeze.resource),
                    })
                    .collect(),
            },
            StakeType::Rewards(_) => vec![TronContract::WithdrawBalance { owner }],
            StakeType::Withdraw(_) => vec![TronContract::WithdrawExpireUnfreeze { owner }],
            StakeType::Freeze(resource) => vec![TronContract::FreezeBalanceV2 {
                owner,
                frozen_balance: input.value_as_u64()?,
                resource: TronResource::from(resource),
            }],
            StakeType::Unfreeze(resource) => vec![TronContract::UnfreezeBalanceV2 {
                owner,
                unfreeze_balance: input.value_as_u64()?,
                resource: TronResource::from(resource),
            }],
        };

        Ok(contracts.into_iter().map(ContractPayload::from).collect())
    })
}

pub(crate) fn sign_data(input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
    validate_sender(input, private_key)?;
    let payload = WalletConnectPayload::parse(input)?;
    let transaction_hash = payload.transaction_hash()?;
    let signature = sign_raw_hash(&transaction_hash, private_key)?;
    payload.into_output(transaction_hash, signature)
}

fn validate_sender(input: &SignerInput, private_key: &[u8]) -> Result<TronAddress, SignerError> {
    let sender = TronAddress::parse(&input.sender_address)?;
    if sender != TronAddress::from_private_key(private_key)? {
        return SignerError::invalid_input_err("Tron sender address does not match private key");
    }
    Ok(sender)
}

fn sign_contract_payload<F>(input: &SignerInput, private_key: &[u8], build_payload: F) -> Result<String, SignerError>
where
    F: FnOnce(TronAddress) -> Result<ContractPayload, SignerError>,
{
    let owner = validate_sender(input, private_key)?;
    let payload = build_payload(owner)?;
    sign_built_contract_payload(input, payload, private_key)
}

fn sign_contract_payloads<F>(input: &SignerInput, private_key: &[u8], build_payloads: F) -> Result<Vec<String>, SignerError>
where
    F: FnOnce(TronAddress) -> Result<Vec<ContractPayload>, SignerError>,
{
    let owner = validate_sender(input, private_key)?;
    build_payloads(owner)?.into_iter().map(|payload| sign_built_contract_payload(input, payload, private_key)).collect()
}

fn sign_built_contract_payload(input: &SignerInput, payload: ContractPayload, private_key: &[u8]) -> Result<String, SignerError> {
    let fee_limit = match payload.contract {
        TronContract::TriggerSmart(_) => input.fee.gas_limit()?,
        _ => input.fee.fee()?,
    };
    let raw_data = TronRawData::from_input_with_data(input, payload.contract, fee_limit, payload.data)?;
    sign_raw_data(raw_data, private_key)
}

fn sign_raw_data(raw_data: TronRawData, private_key: &[u8]) -> Result<String, SignerError> {
    let raw_data_bytes = raw_data.encode();
    let transaction_id = sha256(&raw_data_bytes);
    let signature = sign_raw_hash(&transaction_id, private_key)?;

    serde_json::to_string(&SignedTransactionJson::new(raw_data.json(), &raw_data_bytes, &transaction_id, signature)).map_err(Into::into)
}

fn sign_raw_hash(hash: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
    Ok(hex::encode(Signer::sign_digest(SignatureScheme::Secp256k1, hash, private_key)?))
}
