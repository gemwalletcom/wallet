use num_bigint::BigUint;
use primitives::{
    Asset, SignerError, decode_hex,
    swap::{ApprovalData, SwapQuoteData},
};

use super::{TronContract, TronTriggerSmartContract};
use crate::address::TronAddress;
use crate::trc20;

impl TronTriggerSmartContract {
    pub(crate) fn token_transfer(owner: TronAddress, token_id: &str, destination: &str, value: &BigUint) -> Result<Self, SignerError> {
        let destination = TronAddress::parse(destination)?;
        Ok(Self {
            owner,
            contract: TronAddress::parse(token_id)?,
            data: trc20::encode_transfer(&destination, &value.to_string()).map_err(SignerError::invalid_input)?,
            call_value: None,
            call_token_value: None,
            token_id: None,
        })
    }

    pub(crate) fn token_approval(owner: TronAddress, approval: &ApprovalData) -> Result<Self, SignerError> {
        let contract = TronAddress::parse_hex_or_base58(&approval.token)?;
        let spender = TronAddress::parse_hex_or_base58(&approval.spender)?;
        Ok(Self {
            owner,
            contract,
            data: trc20::encode_approval_max(&spender).map_err(SignerError::invalid_input)?,
            call_value: None,
            call_token_value: None,
            token_id: None,
        })
    }
}

impl TronContract {
    pub(crate) fn contract_swap(owner: TronAddress, from_asset: &Asset, swap_data: &SwapQuoteData) -> Result<Self, SignerError> {
        let call_data = if swap_data.data.is_empty() { Vec::new() } else { decode_hex(&swap_data.data)? };

        if from_asset.id.is_token() || !call_data.is_empty() {
            Self::contract_call_swap(owner, swap_data, call_data)
        } else {
            Self::native_contract_transfer_swap(owner, swap_data)
        }
    }

    fn native_contract_transfer_swap(owner: TronAddress, swap_data: &SwapQuoteData) -> Result<Self, SignerError> {
        let to = TronAddress::parse_hex_or_base58(&swap_data.to)?;
        let amount = u64::try_from(&swap_data.value).map_err(|_| SignerError::invalid_input("Tron swap value is too large"))?;
        Ok(Self::Transfer { owner, to, amount })
    }

    fn contract_call_swap(owner: TronAddress, swap_data: &SwapQuoteData, call_data: Vec<u8>) -> Result<Self, SignerError> {
        if call_data.is_empty() {
            return SignerError::invalid_input_err("Tron contract swap calldata is required");
        }
        let contract = TronAddress::parse_hex_or_base58(&swap_data.to)?;
        let call_value = u64::try_from(&swap_data.value).map_err(|_| SignerError::invalid_input("Tron contract call value is too large"))?;
        Ok(Self::TriggerSmart(TronTriggerSmartContract {
            owner,
            contract,
            data: call_data,
            call_value: (call_value != 0).then_some(call_value),
            call_token_value: None,
            token_id: None,
        }))
    }
}
