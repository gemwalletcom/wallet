use gem_client::{Target, build_path_with_query};
use primitives::chain_cosmos::CosmosChain;

use crate::constants::BOND_STATUS_BONDED;

const VALIDATORS_PAGE_LIMIT: usize = 1000;

#[derive(Clone, Debug)]
pub enum CosmosTarget {
    GetTransaction { hash: String },
    GetBlock { height: String },
    GetTransactions { key: &'static str, filter: String, limit: usize },
    GetValidators,
    GetDelegatorValidators { address: String },
    GetStakingPool,
    GetInflation,
    GetAnnualProvisions,
    GetSupply { denom: String },
    GetOsmosisMintParams,
    GetOsmosisEpochProvisions,
    GetBalances { address: String, page_key: Option<String> },
    GetDelegations { address: String, page_key: Option<String> },
    GetUnbondingDelegations { address: String, page_key: Option<String> },
    GetDelegationRewards { address: String },
    GetAccount { address: String },
    GetNodeInfo,
    GetContractSmartQuery { contract: String, encoded_query: String },
    GetFeemarketGasPrice { denom: String },
    GetBaseFee { chain: CosmosChain },
    BroadcastTransaction,
}

fn page_path(path: &str, page_key: &Option<String>) -> String {
    match page_key {
        Some(key) => build_path_with_query(path, &[("pagination.key", key)]),
        None => path.to_string(),
    }
}

impl Target for CosmosTarget {
    fn path(&self) -> String {
        match self {
            Self::GetTransaction { hash } => format!("/cosmos/tx/v1beta1/txs/{hash}"),
            Self::GetBlock { height } => format!("/cosmos/base/tendermint/v1beta1/blocks/{height}"),
            Self::GetTransactions { key, filter, limit } => format!("/cosmos/tx/v1beta1/txs?{key}={filter}&pagination.limit={limit}&page=1"),
            Self::GetValidators => format!("/cosmos/staking/v1beta1/validators?status={BOND_STATUS_BONDED}&pagination.limit={VALIDATORS_PAGE_LIMIT}"),
            Self::GetDelegatorValidators { address } => format!("/cosmos/staking/v1beta1/delegators/{address}/validators"),
            Self::GetStakingPool => "/cosmos/staking/v1beta1/pool".to_string(),
            Self::GetInflation => "/cosmos/mint/v1beta1/inflation".to_string(),
            Self::GetAnnualProvisions => "/cosmos/mint/v1beta1/annual_provisions".to_string(),
            Self::GetSupply { denom } => format!("/cosmos/bank/v1beta1/supply/by_denom?denom={denom}"),
            Self::GetOsmosisMintParams => "/osmosis/mint/v1beta1/params".to_string(),
            Self::GetOsmosisEpochProvisions => "/osmosis/mint/v1beta1/epoch_provisions".to_string(),
            Self::GetBalances { address, page_key } => page_path(&format!("/cosmos/bank/v1beta1/balances/{address}"), page_key),
            Self::GetDelegations { address, page_key } => page_path(&format!("/cosmos/staking/v1beta1/delegations/{address}"), page_key),
            Self::GetUnbondingDelegations { address, page_key } => page_path(&format!("/cosmos/staking/v1beta1/delegators/{address}/unbonding_delegations"), page_key),
            Self::GetDelegationRewards { address } => format!("/cosmos/distribution/v1beta1/delegators/{address}/rewards"),
            Self::GetAccount { address } => format!("/cosmos/auth/v1beta1/accounts/{address}"),
            Self::GetNodeInfo => "/cosmos/base/tendermint/v1beta1/node_info".to_string(),
            Self::GetContractSmartQuery { contract, encoded_query } => format!("/cosmwasm/wasm/v1/contract/{contract}/smart/{encoded_query}"),
            Self::GetFeemarketGasPrice { denom } => format!("/feemarket/v1/gas_price/{denom}"),
            Self::GetBaseFee { chain } => format!("/{}/txfees/v1beta1/cur_eip_base_fee", chain.as_ref()),
            Self::BroadcastTransaction => "/cosmos/tx/v1beta1/txs".to_string(),
        }
    }
}
