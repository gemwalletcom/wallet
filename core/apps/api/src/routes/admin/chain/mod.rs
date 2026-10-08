mod address;
mod block;
mod defi;
mod fee;
mod nft;
mod node;
mod staking;
mod swap;
mod token;
mod transaction;

use axum::Router;
use axum::routing::get;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/blocks/{chain}/latest", get(block::get_latest_block_number))
        .route("/blocks/{chain}/{block_number}", get(block::get_block_transactions))
        .route("/blocks/{chain}/{block_number}/finalize", get(block::get_block_transactions_finalize))
        .route("/fee-estimates/{chain}", get(fee::get_chain_fee_estimates))
        .route("/nodes/{chain}/status", get(node::get_nodes_status))
        .route("/swaps/quote", get(swap::get_swap_quote))
        .route("/swaps/{provider}/transaction/{hash}", get(swap::get_swap_result))
        .route("/swaps/{provider}/vault_addresses", get(swap::get_vault_addresses))
        .route("/staking/{chain}/validators", get(staking::get_validators))
        .route("/staking/{chain}/apy", get(staking::get_staking_apy))
        .route("/token/{chain}/{token_id}/info", get(token::get_token))
        .route("/address/{chain}/{address}/balances", get(address::get_balances))
        .route("/address/{chain}/{address}/assets", get(address::get_assets))
        .route("/address/{chain}/{address}/transactions", get(address::get_transactions))
        .route("/address/{chain}/{address}/defi/positions", get(defi::get_defi_positions))
        .route("/address/{chain}/{address}/nfts", get(nft::get_nfts))
        .route("/nft/assets/{asset_id}", get(nft::get_nft_asset))
        .route("/nft/collections/{collection_id}", get(nft::get_nft_collection))
        .route("/transactions/{chain}/{hash}", get(transaction::get_transaction))
        .route("/transactions/{chain}/{hash}/status", get(transaction::get_transaction_status))
}
