use gem_hash::sha2::sha256;
use num_bigint::BigUint;
use primitives::{
    Address as _, SignerError, SignerInput, StakeType, TransactionLoadMetadata, TronStakeData, decode_hex,
    swap::{SwapQuoteData, SwapQuoteDataType},
};
use signer::{SignatureScheme, Signer};

use crate::address::TronAddress;
use crate::models::{SignedTransactionJson, TronContract, TronRawData, TronResource, WalletConnectPayload};

const ABI_WORD_LEN: usize = 32;
const TRC20_TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];

struct ContractPayload {
    contract: TronContract,
    fee_limit: u64,
    data: Option<Vec<u8>>,
}

pub(crate) fn sign_transfer(input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
    sign_contract_payload(input, private_key, |owner| {
        build_native_transfer(input, owner, &input.destination_address, input.value_as_u64()?)
    })
}

pub(crate) fn sign_token_transfer(input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
    sign_contract_payload(input, private_key, |owner| build_token_transfer(input, owner, &input.destination_address))
}

pub(crate) fn sign_swap(input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
    let swap = input.input_type.get_swap_data()?;
    let swap_data = &swap.data;
    let from_asset = input.input_type.get_asset();

    let output = sign_contract_payload(input, private_key, |owner| match swap_data.data_type {
        SwapQuoteDataType::Transfer => {
            if from_asset.id.is_token() {
                build_token_transfer(input, owner, &swap_data.to)
            } else {
                build_native_transfer(input, owner, &swap_data.to, input.swap_value_u64()?)
            }
        }
        SwapQuoteDataType::Contract => build_contract_swap(input, owner, swap_data),
    })?;

    Ok(vec![output])
}

fn build_native_transfer(input: &SignerInput, owner: TronAddress, destination: &str, value: u64) -> Result<ContractPayload, SignerError> {
    let contract = TronContract::Transfer {
        owner,
        to: TronAddress::parse(destination)?,
        amount: value,
    };
    let fee_limit = input.fee.fee()?;
    Ok(ContractPayload { contract, fee_limit, data: None })
}

fn build_contract_swap(input: &SignerInput, owner: TronAddress, swap_data: &SwapQuoteData) -> Result<ContractPayload, SignerError> {
    let from_asset = input.input_type.get_asset();
    let call_data = if swap_data.data.is_empty() { Vec::new() } else { decode_hex(&swap_data.data)? };
    let memo = swap_data.memo.as_deref().map(decode_hex).transpose()?;

    if from_asset.id.is_token() || !call_data.is_empty() {
        build_contract_call_swap(input, owner, swap_data, call_data, memo)
    } else {
        build_native_contract_transfer_swap(input, owner, swap_data, memo)
    }
}

fn build_native_contract_transfer_swap(input: &SignerInput, owner: TronAddress, swap_data: &SwapQuoteData, memo: Option<Vec<u8>>) -> Result<ContractPayload, SignerError> {
    let to = TronAddress::from_hex_or_base58(&swap_data.to).ok_or_else(|| SignerError::invalid_input(format!("invalid Tron address: {}", swap_data.to)))?;
    let amount = swap_data.value.parse::<u64>().map_err(|_| SignerError::invalid_input("invalid Tron swap value"))?;
    let contract = TronContract::Transfer { owner, to, amount };
    let fee_limit = input.fee.fee()?;
    Ok(ContractPayload { contract, fee_limit, data: memo })
}

fn build_contract_call_swap(input: &SignerInput, owner: TronAddress, swap_data: &SwapQuoteData, call_data: Vec<u8>, memo: Option<Vec<u8>>) -> Result<ContractPayload, SignerError> {
    if call_data.is_empty() {
        return SignerError::invalid_input_err("Tron contract swap calldata is required");
    }
    let contract_address = TronAddress::from_hex_or_base58(&swap_data.to).ok_or_else(|| SignerError::invalid_input(format!("invalid Tron address: {}", swap_data.to)))?;
    let call_value = swap_data.value.parse::<u64>().map_err(|_| SignerError::invalid_input("invalid Tron contract call value"))?;
    let contract = TronContract::TriggerSmart {
        owner,
        contract: contract_address,
        data: call_data,
        call_value: (call_value != 0).then_some(call_value),
        call_token_value: None,
        token_id: None,
    };
    let fee_limit = input.fee.gas_limit()?;
    Ok(ContractPayload { contract, fee_limit, data: memo })
}

fn build_token_transfer(input: &SignerInput, owner: TronAddress, destination: &str) -> Result<ContractPayload, SignerError> {
    let token_id = input.input_type.get_asset().id.get_token_id()?;
    let destination = TronAddress::parse(destination)?;
    let contract = TronContract::TriggerSmart {
        owner,
        contract: TronAddress::parse(token_id)?,
        data: encode_trc20_transfer(&destination, &input.value)?,
        call_value: None,
        call_token_value: None,
        token_id: None,
    };
    let fee_limit = input.fee.gas_limit()?;
    Ok(ContractPayload { contract, fee_limit, data: None })
}

pub(crate) fn sign_stake(input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
    let stake_type = input.input_type.get_stake_type().map_err(SignerError::invalid_input)?;
    let fee_limit = input.fee.fee()?;
    let TransactionLoadMetadata::Tron { stake_data, .. } = &input.metadata else {
        return SignerError::invalid_input_err("Missing tron metadata");
    };

    sign_contract_payloads(input, private_key, |owner| {
        let contracts = match stake_type {
            StakeType::Stake(_) | StakeType::Redelegate(_) => match stake_data {
                TronStakeData::Votes(votes) => vec![TronContract::vote_witness(owner, votes)?],
                TronStakeData::Unfreeze(_) => return SignerError::invalid_input_err("Expected Tron vote stake data"),
            },
            StakeType::Unstake(_) => match stake_data {
                TronStakeData::Votes(votes) => vec![TronContract::vote_witness(owner, votes)?],
                TronStakeData::Unfreeze(unfreezes) => unfreezes
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

        Ok(contracts.into_iter().map(|contract| ContractPayload { contract, fee_limit, data: None }).collect())
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
    build_payloads(owner)?
        .into_iter()
        .map(|payload| sign_built_contract_payload(input, payload, private_key))
        .collect()
}

fn sign_built_contract_payload(input: &SignerInput, payload: ContractPayload, private_key: &[u8]) -> Result<String, SignerError> {
    let raw_data = TronRawData::from_input_with_data(input, payload.contract, payload.fee_limit, payload.data)?;
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

fn encode_trc20_transfer(destination: &TronAddress, value: &str) -> Result<Vec<u8>, SignerError> {
    let mut data = TRC20_TRANSFER_SELECTOR.to_vec();
    data.extend(pad_left(destination.as_bytes(), ABI_WORD_LEN)?);
    data.extend(pad_left(
        &value.parse::<BigUint>().map_err(|_| SignerError::invalid_input("invalid TRC20 amount"))?.to_bytes_be(),
        ABI_WORD_LEN,
    )?);
    Ok(data)
}

fn pad_left(data: &[u8], len: usize) -> Result<Vec<u8>, SignerError> {
    if data.len() > len {
        return SignerError::invalid_input_err("value does not fit padded length");
    }
    let mut padded = vec![0u8; len - data.len()];
    padded.extend_from_slice(data);
    Ok(padded)
}
