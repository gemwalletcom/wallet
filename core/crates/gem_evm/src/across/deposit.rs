use std::{error::Error, io};

use alloy_primitives::{Address, B256, Bytes, LogData, U256, hex};
use alloy_sol_types::{SolCall, SolEvent, SolValue};

use super::asset::AcrossAsset;
use super::contracts::{
    V3SpokePoolInterface::{FundsDeposited, V3FundsDeposited, depositV3Call},
    multicall_handler,
};
use crate::{contracts::IERC20, u256::u256_to_biguint};
use primitives::{AssetId, Chain, SwapProvider, TransactionSwapMetadata, TransactionSwapReferralFee, swap::EVM_REFERRAL_ADDRESS};
use std::str::FromStr;

pub struct RelayData {
    pub depositor: B256,
    pub recipient: B256,
    pub exclusive_relayer: B256,
    pub input_token: B256,
    pub output_token: B256,
    pub input_amount: U256,
    pub output_amount: U256,
    pub origin_chain_id: U256,
    pub deposit_id: U256,
    pub fill_deadline: u32,
    pub exclusivity_deadline: u32,
    pub message: Bytes,
}

impl RelayData {
    fn referral_fee(&self, to_asset: &AssetId) -> Option<TransactionSwapReferralFee> {
        let amount = self.referral_fee_amount()?;
        Some(TransactionSwapReferralFee {
            asset_id: to_asset.clone(),
            value: u256_to_biguint(&(amount * AcrossAsset::from_asset(to_asset)?.scale)),
        })
    }

    fn user_output_amount(&self) -> Option<U256> {
        match self.referral_fee_amount() {
            Some(fee) => self.output_amount.checked_sub(fee),
            None => Some(self.output_amount),
        }
    }

    fn referral_fee_amount(&self) -> Option<U256> {
        let referral = Address::from_str(EVM_REFERRAL_ADDRESS).ok()?;
        let instructions = multicall_handler::Instructions::abi_decode(&self.message).ok()?;
        instructions.calls.into_iter().find_map(|call| {
            if call.target == referral && !call.value.is_zero() {
                return Some(call.value);
            }
            IERC20::transferCall::abi_decode(&call.callData).ok().filter(|transfer| transfer.to == referral).map(|transfer| transfer.value)
        })
    }
}

pub struct Deposit {
    pub destination_chain_id: u64,
    pub relay_data: RelayData,
}

impl Deposit {
    pub fn swap_metadata(&self, asset_for_token: impl Fn(Chain, Address) -> Option<AssetId>) -> Option<TransactionSwapMetadata> {
        let relay_data = &self.relay_data;
        let from_asset = asset_for_token(Chain::from_chain_id(relay_data.origin_chain_id.to::<u64>())?, Address::from_word(relay_data.input_token))?;
        let to_asset = asset_for_token(Chain::from_chain_id(self.destination_chain_id)?, Address::from_word(relay_data.output_token))?;
        let from_value = u256_to_biguint(&(relay_data.input_amount * AcrossAsset::from_asset(&from_asset)?.scale));
        let to_value = u256_to_biguint(&(relay_data.user_output_amount()? * AcrossAsset::from_asset(&to_asset)?.scale));
        let referral_fee = relay_data.referral_fee(&to_asset);
        Some(TransactionSwapMetadata::new(from_asset, from_value, to_asset, to_value, SwapProvider::Across).with_referral_fee(referral_fee))
    }

    fn from_v3(event: V3FundsDeposited, origin_chain_id: u64) -> Self {
        Self {
            destination_chain_id: event.destinationChainId.to::<u64>(),
            relay_data: RelayData {
                depositor: event.depositor.into_word(),
                recipient: event.recipient.into_word(),
                exclusive_relayer: event.exclusiveRelayer.into_word(),
                input_token: event.inputToken.into_word(),
                output_token: event.outputToken.into_word(),
                input_amount: event.inputAmount,
                output_amount: event.outputAmount,
                origin_chain_id: U256::from(origin_chain_id),
                deposit_id: U256::from(event.depositId),
                fill_deadline: event.fillDeadline,
                exclusivity_deadline: event.exclusivityDeadline,
                message: event.message,
            },
        }
    }

    fn from_universal(event: FundsDeposited, origin_chain_id: u64) -> Self {
        Self {
            destination_chain_id: event.destinationChainId.to::<u64>(),
            relay_data: RelayData {
                depositor: event.depositor,
                recipient: event.recipient,
                exclusive_relayer: event.exclusiveRelayer,
                input_token: event.inputToken,
                output_token: event.outputToken,
                input_amount: event.inputAmount,
                output_amount: event.outputAmount,
                origin_chain_id: U256::from(origin_chain_id),
                deposit_id: event.depositId,
                fill_deadline: event.fillDeadline,
                exclusivity_deadline: event.exclusivityDeadline,
                message: event.message,
            },
        }
    }

    fn from_deposit_v3_call(call: depositV3Call, origin_chain_id: u64) -> Self {
        Self {
            destination_chain_id: call.destinationChainId.saturating_to::<u64>(),
            relay_data: RelayData {
                depositor: call.depositor.into_word(),
                recipient: call.recipient.into_word(),
                exclusive_relayer: call.exclusiveRelayer.into_word(),
                input_token: call.inputToken.into_word(),
                output_token: call.outputToken.into_word(),
                input_amount: call.inputAmount,
                output_amount: call.outputAmount,
                origin_chain_id: U256::from(origin_chain_id),
                deposit_id: U256::ZERO,
                fill_deadline: call.fillDeadline,
                exclusivity_deadline: call.exclusivityDeadline,
                message: call.message,
            },
        }
    }
}

pub fn parse_deposit_call(input: &[u8], origin_chain_id: u64) -> Result<Deposit, Box<dyn Error + Send + Sync>> {
    Ok(Deposit::from_deposit_v3_call(depositV3Call::abi_decode(input)?, origin_chain_id))
}

pub fn parse_deposit<'a>(logs: impl IntoIterator<Item = (&'a [String], &'a str)>, origin_chain_id: u64) -> Result<Option<Deposit>, Box<dyn Error + Send + Sync>> {
    for (topics, data) in logs {
        let Some(topic) = topics.first().map(|topic| parse_topic(topic)).transpose()? else {
            continue;
        };
        if topic == V3FundsDeposited::SIGNATURE_HASH {
            let event = V3FundsDeposited::decode_log_data(&alloy_log_data(topics, data)?)?;
            return Ok(Some(Deposit::from_v3(event, origin_chain_id)));
        }
        if topic == FundsDeposited::SIGNATURE_HASH {
            let event = FundsDeposited::decode_log_data(&alloy_log_data(topics, data)?)?;
            return Ok(Some(Deposit::from_universal(event, origin_chain_id)));
        }
    }

    Ok(None)
}

fn parse_topic(topic: &str) -> Result<B256, Box<dyn Error + Send + Sync>> {
    Ok(topic.parse().or_else(|_| format!("0x{topic}").parse())?)
}

fn alloy_log_data(topics: &[String], data: &str) -> Result<LogData, Box<dyn Error + Send + Sync>> {
    let topics = topics.iter().map(|topic| parse_topic(topic)).collect::<Result<Vec<_>, _>>()?;
    let data = Bytes::from(hex::decode(data)?);
    LogData::new(topics, data).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid event topics").into())
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, Bytes, U256, hex};
    use alloy_sol_types::SolEvent;

    use super::{V3FundsDeposited, parse_deposit};

    #[test]
    fn test_parse_v3_deposit() {
        let event = V3FundsDeposited {
            inputToken: Address::ZERO,
            outputToken: Address::ZERO,
            inputAmount: U256::from(1_000_000),
            outputAmount: U256::from(999_000),
            destinationChainId: U256::from(8453),
            depositId: 123,
            quoteTimestamp: 1_700_000_000,
            fillDeadline: 1_700_003_600,
            exclusivityDeadline: 0,
            depositor: Address::ZERO,
            recipient: Address::ZERO,
            exclusiveRelayer: Address::ZERO,
            message: Bytes::new(),
        };
        let encoded = event.encode_log_data();
        let (topics, data) = encoded.split();
        let topics = topics.into_iter().map(hex::encode).collect::<Vec<_>>();
        let data = hex::encode(data);
        let deposit = parse_deposit([(topics.as_slice(), data.as_str())], 137).unwrap().unwrap();

        assert_eq!(deposit.destination_chain_id, 8453);
        assert_eq!(deposit.relay_data.origin_chain_id, U256::from(137));
        assert_eq!(deposit.relay_data.deposit_id, U256::from(123));
        assert_eq!(deposit.relay_data.input_amount, U256::from(1_000_000));
        assert_eq!(deposit.relay_data.output_amount, U256::from(999_000));
        assert!(parse_deposit([], 137).unwrap().is_none());
    }
}
