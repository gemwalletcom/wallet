use std::error::Error;

use async_trait::async_trait;
use num_bigint::BigInt;
use primitives::{AssetBalance, AssetId, Chain, Transaction, TransactionFee, TransactionLoadInput};

use crate::rpc::parsers::{ParseContext, ProtocolParser};
use crate::rpc::{EvmFeeCalculator, EvmStakingClient};
use crate::transaction_params::TransactionParams;

pub struct MockChainProvider;

struct MockProtocolParser;

impl ProtocolParser for MockProtocolParser {
    fn matches(&self, _context: &ParseContext<'_>) -> bool {
        false
    }

    fn parse(&self, _context: &ParseContext<'_>) -> Option<Transaction> {
        None
    }
}

#[async_trait]
impl EvmStakingClient for MockChainProvider {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        Ok(Some(42.0))
    }

    async fn get_staking_balance(&self, _address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(Some(AssetBalance::new(AssetId::from_chain(Chain::SmartChain), 123u32.into())))
    }

    fn protocol_parser(&self) -> Option<&'static dyn ProtocolParser> {
        Some(&MockProtocolParser)
    }
}

#[async_trait]
impl EvmFeeCalculator for MockChainProvider {
    async fn calculate_fee(&self, _input: &TransactionLoadInput, _params: &TransactionParams, _gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
        Err("MockChainProvider does not calculate fees".into())
    }
}
