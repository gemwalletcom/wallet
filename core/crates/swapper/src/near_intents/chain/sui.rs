use gem_sui::{SuiClient, build_transfer_message_bytes};

use crate::{SwapperError, SwapperQuoteAsset, near_intents::model::DepositData};

pub(crate) async fn build_deposit_data(client: &SuiClient, from_asset: &SwapperQuoteAsset, wallet_address: &str, deposit_address: &str, amount_in: &str) -> Result<DepositData, SwapperError> {
    let amount = amount_in.parse::<u64>().map_err(|_| SwapperError::ComputeQuoteError("Invalid Sui amount provided for deposit".into()))?;

    let message_bytes = build_transfer_message_bytes(client, wallet_address, deposit_address, amount, from_asset.asset_id().token_id.as_deref())
        .await
        .map_err(|error| SwapperError::TransactionError(format!("Failed to build Sui deposit data: {error}")))?;

    Ok(DepositData {
        to: deposit_address.to_string(),
        value: amount_in.to_string(),
        data: message_bytes,
        memo: None,
    })
}
