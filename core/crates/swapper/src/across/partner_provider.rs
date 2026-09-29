use std::collections::BTreeSet;
use std::str::FromStr;

use alloy_primitives::{Address, U256, hex};
use alloy_sol_types::{SolCall, SolValue};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use gem_client::{Client, ClientExt};
use gem_evm::{
    across::{asset::AcrossAsset, contracts::multicall_handler::Instructions, deployment::AcrossDeployment},
    contracts::erc20::IERC20,
    u256::u256_to_biguint,
};
use num_traits::ToPrimitive;
use primitives::{
    AssetId, Chain, SwapProvider,
    swap::{SwapPartnerTransaction, SwapReferralFee, SwapStatus},
};

use super::{
    model::{AcrossDeposit, AcrossDepositsQuery, AcrossPartnerCursor},
    target::AcrossTarget,
};
use crate::{
    SwapperError,
    fees::default_referral_fees,
    partner::{SwapPartnerCursor, SwapPartnerProvider, SwapPartnerTransactionsPage},
};

const DEPOSITS_LIMIT: usize = 100;
const LATEST_LOOKBACK: Duration = Duration::days(1);

pub struct AcrossPartnerProvider<C: Client> {
    client: C,
    fee_address: String,
    handlers: Vec<String>,
}

impl<C: Client> AcrossPartnerProvider<C> {
    pub fn new(client: C) -> Self {
        let handlers = Chain::all()
            .into_iter()
            .filter_map(|chain| AcrossDeployment::deployment_by_chain(&chain))
            .map(|deployment| deployment.multicall_handler().to_string())
            .collect::<BTreeSet<_>>();
        Self {
            client,
            fee_address: default_referral_fees().evm.address,
            handlers: handlers.into_iter().collect(),
        }
    }
}

struct Payout {
    recipient: Address,
    user_amount: U256,
    fee_amount: U256,
}

fn map_payout(message: &str, fee_address: &str) -> Option<Payout> {
    let instructions = Instructions::abi_decode(&hex::decode(message).ok()?).ok()?;
    let amount_to = |recipient: &dyn Fn(&Address) -> bool| {
        instructions.calls.iter().find_map(|call| match IERC20::transferCall::abi_decode(&call.callData) {
            Ok(transfer) => recipient(&transfer.to).then_some(transfer.value),
            Err(_) => (call.callData.is_empty() && recipient(&call.target)).then_some(call.value),
        })
    };
    Some(Payout {
        recipient: instructions.fallbackRecipient,
        fee_amount: amount_to(&|address| address.to_string().eq_ignore_ascii_case(fee_address))?,
        user_amount: amount_to(&|address| *address == instructions.fallbackRecipient)?,
    })
}

fn map_status(status: &str) -> SwapStatus {
    match status {
        "filled" => SwapStatus::Completed,
        "refunded" => SwapStatus::Refunded,
        _ => SwapStatus::Pending,
    }
}

struct DepositAsset {
    asset_id: AssetId,
    scale: U256,
    decimals: u32,
}

fn map_asset(chain_id: u64, token: &str) -> Option<DepositAsset> {
    let asset_id = AcrossDeployment::supported_asset_for_token(Chain::from_chain_id(chain_id)?, Address::from_str(token).ok()?)?;
    let routed = AcrossAsset::from_asset(&asset_id)?;
    let decimals = AcrossDeployment::asset_mappings().into_iter().find(|mapping| mapping.set.contains(&routed.asset_id))?.capital_cost.decimals;
    Some(DepositAsset { asset_id, scale: routed.scale, decimals })
}

fn map_amount_usd(amount: U256, price_usd: Option<&String>, decimals: u32) -> Option<f64> {
    Some(u256_to_biguint(&amount).to_f64()? / 10f64.powi(i32::try_from(decimals).ok()?) * price_usd?.parse::<f64>().ok()?)
}

pub fn map_partner_transaction(deposit: &AcrossDeposit, fee_address: &str) -> Option<SwapPartnerTransaction> {
    let payout = map_payout(&deposit.message, fee_address)?;
    let from = map_asset(deposit.origin_chain_id, &deposit.input_token)?;
    let to = map_asset(deposit.destination_chain_id, &deposit.output_token)?;
    let input_amount = U256::from_str(&deposit.input_amount).ok()?;
    Some(SwapPartnerTransaction {
        provider: SwapProvider::Across,
        provider_transaction_id: deposit.deposit_tx_hash.clone(),
        status: map_status(&deposit.status),
        from_address: deposit.depositor.clone(),
        to_address: payout.recipient.to_checksum(None),
        from_asset_id: from.asset_id,
        from_value: (input_amount * from.scale).to_string(),
        from_amount_usd: map_amount_usd(input_amount, deposit.input_price_usd.as_ref(), from.decimals),
        to_asset_id: to.asset_id.clone(),
        to_value: (payout.user_amount * to.scale).to_string(),
        to_amount_usd: map_amount_usd(payout.user_amount, deposit.output_price_usd.as_ref(), to.decimals),
        referral_fee: Some(SwapReferralFee {
            asset_id: to.asset_id,
            value: (payout.fee_amount * to.scale).to_string(),
            amount_usd: map_amount_usd(payout.fee_amount, deposit.output_price_usd.as_ref(), to.decimals),
        }),
        from_transaction_hash: Some(deposit.deposit_tx_hash.clone()),
        to_transaction_hash: deposit.fill_tx.clone(),
    })
}

fn map_next_cursor(cursor: AcrossPartnerCursor, deposits: &[AcrossDeposit], handlers: usize) -> Result<SwapPartnerCursor, SwapperError> {
    let reached_since = deposits.last().zip(cursor.since).is_some_and(|(deposit, since)| deposit.deposit_block_timestamp.timestamp() < since);
    let next = if deposits.len() == DEPOSITS_LIMIT && !reached_since {
        Some(AcrossPartnerCursor {
            skip: cursor.skip + DEPOSITS_LIMIT,
            ..cursor
        })
    } else if cursor.handler + 1 < handlers {
        Some(AcrossPartnerCursor {
            handler: cursor.handler + 1,
            skip: 0,
            ..cursor
        })
    } else {
        None
    };
    Ok(match next {
        Some(next) => SwapPartnerCursor::Next(serde_json::to_string(&next)?),
        None => SwapPartnerCursor::Latest(serde_json::to_string(&AcrossPartnerCursor {
            since: cursor.walk_started_at.map(|started_at| started_at - LATEST_LOOKBACK.num_seconds()),
            ..AcrossPartnerCursor::default()
        })?),
    })
}

#[async_trait]
impl<C: Client> SwapPartnerProvider for AcrossPartnerProvider<C> {
    fn name(&self) -> &str {
        SwapProvider::Across.id()
    }

    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError> {
        let cursor: AcrossPartnerCursor = cursor.map(|cursor| serde_json::from_str(&cursor)).transpose()?.unwrap_or_default();
        let cursor = AcrossPartnerCursor {
            walk_started_at: cursor.walk_started_at.or(Some(Utc::now().timestamp())),
            ..cursor
        };
        let query = AcrossDepositsQuery {
            recipient: self.handlers.get(cursor.handler).ok_or(SwapperError::InvalidRoute)?.clone(),
            limit: DEPOSITS_LIMIT,
            skip: cursor.skip,
        };
        let deposits: Vec<AcrossDeposit> = self.client.get(AcrossTarget::Deposits { query }).await.map_err(SwapperError::from)?;
        Ok(SwapPartnerTransactionsPage {
            transactions: deposits.iter().filter_map(|deposit| map_partner_transaction(deposit, &self.fee_address)).collect(),
            cursor: map_next_cursor(cursor, &deposits, self.handlers.len())?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_partner_transaction() {
        let deposits: Vec<AcrossDeposit> = serde_json::from_str(include_str!("testdata/deposits.json")).unwrap();
        let fee_address = default_referral_fees().evm.address;
        let summary = deposits
            .iter()
            .map(|deposit| {
                map_partner_transaction(deposit, &fee_address).map(|transaction| {
                    let fee = transaction.referral_fee.unwrap();
                    (transaction.from_asset_id, transaction.from_value, transaction.to_asset_id, transaction.to_value, fee.asset_id, fee.value)
                })
            })
            .collect::<Vec<_>>();
        let arbitrum_usdc = AssetId::from_token(Chain::Arbitrum, "0xaf88d065e77c8cC2239327C5EDb3A432268e5831");

        assert_eq!(
            summary,
            vec![
                Some((
                    AssetId::from_chain(Chain::Ethereum),
                    "15102699778500".to_string(),
                    AssetId::from_chain(Chain::Base),
                    "14309283570437".to_string(),
                    AssetId::from_chain(Chain::Base),
                    "71905947590".to_string()
                )),
                Some((
                    AssetId::from_token(Chain::Hyperliquid, "0xb88339CB7199b77E23DB6E890353E22632Ba630f"),
                    "591009432".to_string(),
                    arbitrum_usdc.clone(),
                    "587876851".to_string(),
                    arbitrum_usdc,
                    "2954155".to_string()
                )),
                None,
            ]
        );
    }
}
