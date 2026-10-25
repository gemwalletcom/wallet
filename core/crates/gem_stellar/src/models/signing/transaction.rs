use super::asset::StellarAssetData;
use super::operation::{Memo, Operation};
use crate::address::StellarAddress;
use crate::models::swap::StellarSwapData;
use num_traits::ToPrimitive;
use primitives::{
    Address, AssetId, SignerError, SignerInput, TransactionSwapReferralFee,
    swap::{STELLAR_REFERRAL_ADDRESS, STELLAR_REFERRAL_FEE_ASSET_IDS},
};
use signer::InvalidInput;

const MEMO_TEXT_MAX_BYTES: usize = 28;

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
        let (from_asset, to_asset) = (&input.input_type.get_asset().id, &input.input_type.get_recipient_asset().id);
        let fee_payment = swap.fee.as_ref().map(|fee| fee_payment(fee, from_asset, to_asset)).transpose()?;
        let input_fee = match &swap.fee {
            Some(fee) if fee.asset_id == *from_asset => fee.value.to_u64().invalid_input("invalid Stellar fee amount")?,
            _ => 0,
        };
        if swap.send_amount.checked_add(input_fee) != Some(input.value_as_u64()?) {
            return SignerError::invalid_input_err("Stellar swap amount does not match the amount paid");
        }
        let account = StellarAddress::from_str(&input.sender_address).invalid_input("invalid Stellar address")?;
        let destination_asset = StellarAssetData::from_asset_id(to_asset)?;
        let change_trust = destination_asset.clone().filter(|_| swap.change_trust).map(|asset| Operation::ChangeTrust { asset });
        let path_payment = Operation::PathPaymentStrictSend {
            send_asset: StellarAssetData::from_asset_id(from_asset)?,
            send_amount: swap.send_amount,
            destination: account.clone(),
            destination_asset,
            destination_min: swap.destination_min,
            path: swap.path.iter().map(StellarAssetData::from_asset_id).collect::<Result<_, _>>()?,
        };

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

fn fee_payment(fee: &TransactionSwapReferralFee, from_asset: &AssetId, to_asset: &AssetId) -> Result<Operation, SignerError> {
    if (fee.asset_id != *from_asset && fee.asset_id != *to_asset) || !STELLAR_REFERRAL_FEE_ASSET_IDS.contains(&fee.asset_id) {
        return SignerError::invalid_input_err("Stellar fee must be paid in the asset sold or bought that Gem's fee account receives");
    }
    Ok(Operation::Payment {
        destination: StellarAddress::from_str(STELLAR_REFERRAL_ADDRESS).invalid_input("invalid Stellar fee address")?,
        asset: StellarAssetData::from_asset_id(&fee.asset_id)?,
        amount: fee.value.to_u64().invalid_input("invalid Stellar fee amount")?,
    })
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
