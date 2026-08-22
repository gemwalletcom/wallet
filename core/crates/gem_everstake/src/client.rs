use std::{error::Error, str::FromStr};

use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use gem_client::Client;
#[cfg(feature = "reqwest")]
use gem_client::{ClientExt, ReqwestClient};
use gem_evm::multicall3::{IMulticall3, create_call3, decode_call3_return};
use gem_evm::rpc::EthereumClient;
use gem_evm::u256::u256_to_biguint;
use num_bigint::BigUint;
use num_traits::Zero;

use crate::constants::EVERSTAKE_ACCOUNTING_ADDRESS;
use crate::contracts::IAccounting;
use crate::models::AccountState;
#[cfg(feature = "reqwest")]
use crate::models::StatsResponse;

#[cfg(feature = "reqwest")]
pub async fn staking_apy(url: &str) -> Result<Option<f64>, Box<dyn Error + Send + Sync>> {
    let client = ReqwestClient::new(url.to_string(), gem_client::reqwest_client());
    let response: StatsResponse = client.get("/api/v1/stats").await?;

    Ok(Some(response.apr * 100.0))
}

pub async fn account_state<C: Client + Clone>(client: &EthereumClient<C>, address: &str) -> Result<AccountState, Box<dyn Error + Sync + Send>> {
    let account = Address::from_str(address).map_err(|e| Box::new(e) as Box<dyn Error + Sync + Send>)?;

    let calls = vec![
        create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::depositedBalanceOfCall { account }),
        create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::pendingBalanceOfCall { account }),
        create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::pendingDepositedBalanceOfCall { account }),
        create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::withdrawRequestCall { staker: account }),
        create_call3(EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::restakedRewardOfCall { account }),
    ];

    let expected = calls.len();
    let results = client.multicall3(calls).await?;
    if results.len() != expected {
        return Err("Unexpected number of multicall results".into());
    }

    Ok(AccountState {
        deposited_balance: decode_balance_result::<IAccounting::depositedBalanceOfCall>(&results[0])?,
        pending_balance: decode_balance_result::<IAccounting::pendingBalanceOfCall>(&results[1])?,
        pending_deposited_balance: decode_balance_result::<IAccounting::pendingDepositedBalanceOfCall>(&results[2])?,
        withdraw_request: decode_call3_return::<IAccounting::withdrawRequestCall>(&results[3])?,
        restaked_reward: decode_balance_result::<IAccounting::restakedRewardOfCall>(&results[4])?,
    })
}

fn decode_balance_result<T: SolCall>(result: &IMulticall3::Result) -> Result<BigUint, Box<dyn Error + Sync + Send>>
where
    T::Return: Into<U256>,
{
    if !result.success {
        return Ok(BigUint::zero());
    }
    let value: U256 = decode_call3_return::<T>(result)?.into();
    Ok(u256_to_biguint(&value))
}
