use super::model::{PartnerSwap, PoolRoute};
use crate::{Quote, SwapperError, SwapperQuoteData, fees::ReferralFee};
use gem_aptos::{AccountAddress, AptosClient, GAS_LIMIT_MULTIPLIER};
use gem_client::Client;
use num_bigint::BigUint;

pub(super) async fn build_quote_data<C: Client>(client: &AptosClient<C>, quote: &Quote, route: &PoolRoute, referral_fee: &ReferralFee) -> Result<SwapperQuoteData, SwapperError> {
    let wallet = quote.request.wallet_address.as_str();
    let swap = PartnerSwap::new(quote, route, referral_fee)?;
    let payload = swap.payload();

    let (account, gas_price) = futures::try_join!(client.get_account(wallet), client.get_gas_price()).map_err(SwapperError::transaction_error)?;
    let simulation = client
        .get_simulated_transaction(wallet, account.sequence_number, payload.clone(), &gas_price.gas_estimate.to_string())
        .await
        .map_err(SwapperError::transaction_error)?;
    let gas_used = swap.verify_simulation(&AccountAddress::from_hex(wallet).map_err(SwapperError::transaction_error)?, &simulation)?;
    let gas_limit = gas_used.saturating_mul(GAS_LIMIT_MULTIPLIER) / 100;

    Ok(SwapperQuoteData::new_contract(String::new(), BigUint::ZERO, serde_json::to_string(&payload)?, None, Some(gas_limit.to_string())))
}
