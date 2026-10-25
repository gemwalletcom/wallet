use super::asset::StellarAssetData;
use super::operation::{Memo, Operation};
use crate::address::StellarAddress;
use crate::models::swap::{StellarSwapData, StellarSwapFee};
use num_traits::ToPrimitive;
use primitives::{Address, SignerError, SignerInput, swap::STELLAR_REFERRAL_TRUSTED_ASSETS};
use signer::InvalidInput;

const MEMO_TEXT_MAX_BYTES: usize = 28;
const PATH_MAX_LENGTH: usize = 5;

#[derive(Clone)]
pub struct StellarTransaction {
    pub account: StellarAddress,
    pub fee: u32,
    pub sequence: u64,
    pub memo: Memo,
    pub time_bounds: Option<u64>,
    pub operations: Vec<Operation>,
}

impl StellarTransaction {
    pub fn transfer(input: &SignerInput) -> Result<Self, SignerError> {
        let amount = input.value_as_u64()?;
        let destination = StellarAddress::from_str(&input.destination_address).invalid_input("invalid Stellar address")?;
        let is_destination_exist = input.metadata.get_is_destination_address_exist()?;

        let operation = if is_destination_exist {
            Operation::Payment { destination, asset: None, amount }
        } else {
            Operation::CreateAccount { destination, amount }
        };

        Self::build(input, fee_u32(input)?, operation)
    }

    pub fn token_transfer(input: &SignerInput) -> Result<Self, SignerError> {
        if !input.metadata.get_is_destination_address_exist()? {
            return SignerError::invalid_input_err("Stellar destination account not found for token transfer");
        }

        let amount = input.value_as_u64()?;
        let operation = Operation::Payment {
            destination: StellarAddress::from_str(&input.destination_address).invalid_input("invalid Stellar address")?,
            asset: Some(StellarAssetData::from_input(input)?),
            amount,
        };

        Self::build(input, fee_u32(input)?, operation)
    }

    pub fn account_action(input: &SignerInput) -> Result<Self, SignerError> {
        let operation = Operation::ChangeTrust { asset: StellarAssetData::from_input(input)? };

        Self::build(input, fee_u32(input)?, operation)
    }

    pub fn swap(input: &SignerInput) -> Result<Self, SignerError> {
        let swap: StellarSwapData = serde_json::from_str(&input.input_type.get_swap_data()?.data.data).invalid_input("invalid Stellar swap data")?;
        if swap.path.len() > PATH_MAX_LENGTH {
            return SignerError::invalid_input_err("Stellar path payment path is too long");
        }
        let account = StellarAddress::from_str(&input.sender_address).invalid_input("invalid Stellar address")?;
        let destination_asset = StellarAssetData::from_asset_id(&input.input_type.get_recipient_asset().id)?;
        let change_trust = change_trust(swap.change_trust, destination_asset.as_ref())?;
        let path_payment = Operation::PathPaymentStrictSend {
            send_asset: StellarAssetData::from_asset_id(&input.input_type.get_asset().id)?,
            send_amount: swap.send_amount,
            destination: account.clone(),
            destination_asset,
            destination_min: swap.destination_min,
            path: swap.path.iter().map(StellarAssetData::from_asset_id).collect::<Result<_, _>>()?,
        };
        let fee_payment = fee_payment(swap.fee.as_ref())?;

        Ok(Self {
            account,
            fee: fee_u32(input)?,
            sequence: input.metadata.get_sequence()?,
            memo: Memo::None,
            time_bounds: Some(swap.max_time),
            operations: change_trust.into_iter().chain([path_payment]).chain(fee_payment).collect(),
        })
    }

    fn build(input: &SignerInput, fee: u32, operation: Operation) -> Result<Self, SignerError> {
        Ok(Self {
            account: StellarAddress::from_str(&input.sender_address).invalid_input("invalid Stellar address")?,
            fee,
            sequence: input.metadata.get_sequence()?,
            memo: memo(input.memo.as_deref())?,
            time_bounds: None,
            operations: vec![operation],
        })
    }
}

fn change_trust(is_needed: bool, asset: Option<&StellarAssetData>) -> Result<Option<Operation>, SignerError> {
    match (is_needed, asset) {
        (true, Some(asset)) => Ok(Some(Operation::ChangeTrust { asset: asset.clone() })),
        (true, None) => SignerError::invalid_input_err("Stellar trustline requires a token"),
        (false, _) => Ok(None),
    }
}

fn fee_payment(fee: Option<&StellarSwapFee>) -> Result<Option<Operation>, SignerError> {
    let Some(fee) = fee else {
        return Ok(None);
    };
    if !fee.asset.is_native() && !STELLAR_REFERRAL_TRUSTED_ASSETS.contains(&fee.asset) {
        return SignerError::invalid_input_err("Stellar fee asset is not trusted by the fee account");
    }
    Ok(Some(Operation::Payment {
        destination: StellarAddress::from_str(&fee.destination).invalid_input("invalid Stellar fee address")?,
        asset: StellarAssetData::from_asset_id(&fee.asset)?,
        amount: fee.amount,
    }))
}

fn fee_u32(input: &SignerInput) -> Result<u32, SignerError> {
    input.fee.fee.to_u32().invalid_input("invalid transaction fee")
}

fn memo(value: Option<&str>) -> Result<Memo, SignerError> {
    match value {
        Some(text) if text.len() > MEMO_TEXT_MAX_BYTES => SignerError::invalid_input_err("Stellar memo text must be at most 28 bytes"),
        Some(text) => Ok(Memo::Text(text.to_string())),
        None => Ok(Memo::None),
    }
}
