use std::error::Error;

use etherscan::GasOracle;
use gem_evm::ether_conv::EtherConv;
use gem_evm::fee_calculator::{get_fee_history_blocks, get_reward_percentiles};
use gem_evm::jsonrpc::EthereumRpc;
use gem_evm::models::fee::EthereumFeeHistory;
use gem_evm::provider::preload_mapper::map_transaction_fee_rates;
use gem_jsonrpc::alien::RpcProvider;
use gemstone::alien::{new_alien_client, reqwest_provider::NativeProvider};
use gemstone::network::JsonRpcClient;
use num_bigint::BigInt;
use primitives::{EVMChain, PriorityFeeValue, fee::FeePriority};
use std::sync::Arc;

const WEI_PER_GWEI: f64 = 1_000_000_000.0;

#[derive(Debug)]
pub struct GemstoneFeeData {
    pub latest_block: u64,
    pub suggest_base_fee: String,
    pub gas_used_ratio: Option<String>,
    pub priority_fees: Vec<PriorityFeeValue>,
}

impl From<GasOracle> for GemstoneFeeData {
    fn from(oracle: GasOracle) -> Self {
        Self {
            latest_block: oracle.last_block,
            suggest_base_fee: oracle.suggest_base_fee.to_string(),
            gas_used_ratio: oracle.gas_used_ratio.split(',').next_back().and_then(|ratio| ratio.trim().parse().ok()).map(format_gas_used_ratio),
            priority_fees: vec![
                PriorityFeeValue {
                    priority: FeePriority::Normal,
                    value: gwei_to_wei(oracle.propose_gas_price - oracle.suggest_base_fee),
                },
                PriorityFeeValue {
                    priority: FeePriority::Fast,
                    value: gwei_to_wei(oracle.fast_gas_price - oracle.suggest_base_fee),
                },
            ],
        }
    }
}

#[derive(Debug)]
pub struct GemstoneClient {
    native_provider: Arc<NativeProvider>,
    chain: EVMChain,
}

impl GemstoneClient {
    pub fn new(native_provider: Arc<NativeProvider>, chain: EVMChain) -> Self {
        Self { native_provider, chain }
    }

    pub async fn get_fee_data(&self) -> Result<GemstoneFeeData, Box<dyn Error + Send + Sync>> {
        let endpoint = self.native_provider.get_endpoint(self.chain.to_chain())?;
        let client = JsonRpcClient::new(new_alien_client(endpoint, self.native_provider.clone()));
        let blocks = get_fee_history_blocks(self.chain);
        let call = EthereumRpc::FeeHistory {
            blocks,
            reward_percentiles: get_reward_percentiles(self.chain).to_vec(),
        };

        let fee_history: EthereumFeeHistory = client.request(call).await?;
        let rates = map_transaction_fee_rates(self.chain, &fee_history)?;
        let base_fee = rates.first().ok_or("Missing fee rates")?.gas_price_type.gas_price();

        Ok(GemstoneFeeData {
            latest_block: fee_history.oldest_block + blocks - 1,
            suggest_base_fee: EtherConv::to_gwei(&base_fee),
            gas_used_ratio: fee_history.gas_used_ratio.last().copied().map(format_gas_used_ratio),
            priority_fees: rates
                .into_iter()
                .map(|rate| PriorityFeeValue {
                    priority: rate.priority,
                    value: rate.gas_price_type.priority_fee(),
                })
                .collect(),
        })
    }
}

fn gwei_to_wei(gwei: f64) -> BigInt {
    BigInt::from((gwei * WEI_PER_GWEI).round() as i64)
}

fn format_gas_used_ratio(ratio: f64) -> String {
    format!("{:.1}%", ratio * 100.0)
}
