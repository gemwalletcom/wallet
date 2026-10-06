use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use serde::{Deserialize, Serialize};

use crate::{
    SwapperError,
    bridgers::asset::{Network, get_token_address},
    fees::DEFAULT_REFERRER,
};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub source_flag: String,
    pub from_token_address: String,
    pub to_token_address: String,
    #[serde(with = "serde_serializers::biguint::string")]
    pub from_token_amount: BigUint,
    pub from_token_chain: String,
    pub to_token_chain: String,
}

impl QuoteRequest {
    pub fn new(request: &crate::QuoteRequest, from_token_amount: BigUint) -> Result<Self, SwapperError> {
        Ok(Self {
            source_flag: DEFAULT_REFERRER.to_string(),
            from_token_address: get_token_address(&request.from_asset.asset_id())?,
            to_token_address: get_token_address(&request.to_asset.asset_id())?,
            from_token_amount,
            from_token_chain: Network::from_source_chain(request.from_asset.chain())?.code.to_string(),
            to_token_chain: Network::from_chain(request.to_asset.chain())?.code.to_string(),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteData {
    pub tx_data: QuoteTxData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteTxData {
    #[serde(with = "serde_serializers::biguint::string")]
    pub amount_out_min: BigUint,
    pub to_token_amount: String,
    pub deposit_min: String,
    pub deposit_max: String,
    pub chain_fee: String,
    pub estimated_time: u32,
}

impl QuoteTxData {
    pub fn eta_in_seconds(&self) -> Option<u32> {
        match self.estimated_time {
            1 => Some(180),
            2 | 10 => Some(600),
            3 => Some(1800),
            _ => None,
        }
    }

    pub fn get_deposit_range(&self, decimals: u32) -> Result<(BigUint, BigUint), SwapperError> {
        Ok((
            BigNumberFormatter::value_from_amount_biguint(&self.deposit_min, decimals)?,
            BigNumberFormatter::value_from_amount_biguint(&self.deposit_max, decimals)?,
        ))
    }

    pub fn get_to_value(&self, decimals: u32) -> Result<BigUint, SwapperError> {
        let to_amount = BigNumberFormatter::value_from_amount_biguint(&self.to_token_amount, decimals)?;
        let chain_fee = BigNumberFormatter::value_from_amount_biguint(&self.chain_fee, decimals)?;
        if to_amount <= chain_fee {
            return Err(SwapperError::NoQuoteAvailable);
        }
        Ok(to_amount - chain_fee)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteData {
    #[serde(with = "serde_serializers::biguint::string")]
    pub amount_out_min: BigUint,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_values() {
        let quote = QuoteTxData {
            amount_out_min: BigUint::ZERO,
            to_token_amount: "0.599919".to_string(),
            deposit_min: "147.8".to_string(),
            deposit_max: "20000".to_string(),
            chain_fee: "0.0001".to_string(),
            estimated_time: 10,
        };

        assert_eq!(quote.get_deposit_range(18).unwrap(), (BigUint::from(147_800_000_000_000_000_000u128), BigUint::from(20_000_000_000_000_000_000_000u128)));
        assert_eq!(quote.get_to_value(8).unwrap(), BigUint::from(59_981_900u64));
        assert_eq!(QuoteTxData { chain_fee: "1".to_string(), ..quote }.get_to_value(8).unwrap_err(), SwapperError::NoQuoteAvailable);
    }
}
