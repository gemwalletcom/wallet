use std::time::Duration;

use chain_traits::node_check::NodeCheckRecorder;
use gem_client::Client;

use crate::rpc::EthereumProvider;

pub(super) async fn record_receipt_checks<C: Client + Clone>(provider: &EthereumProvider<C>, recorder: NodeCheckRecorder, latest_block: u64) -> NodeCheckRecorder {
    let recorder = recorder
        .record_timed(
            "eth_getReceipt(history)",
            get_receipt_at_age(provider, latest_block, Duration::from_secs(24 * 60 * 60), "1 day"),
        )
        .await;

    recorder
        .record_optional_timed(
            "eth_getReceipt(archive)",
            get_receipt_at_age(provider, latest_block, Duration::from_secs(90 * 24 * 60 * 60), "3 months"),
        )
        .await
}

async fn find_transaction_hash<C: Client + Clone>(provider: &EthereumProvider<C>, latest_block: u64, block_depth: u64) -> Result<String, String> {
    let target_block = latest_block.saturating_sub(block_depth);
    let block_time = u64::from(provider.get_chain().block_time()).max(1);
    let one_minute_ms = 60_000_u64;
    let search_blocks = one_minute_ms.div_ceil(block_time).clamp(1, 10);

    for offset in 0..search_blocks {
        let block_number = target_block.saturating_sub(offset);
        let block = provider.get_block(block_number).await.map_err(|error| error.to_string())?;
        if let Some(transaction) = block.transactions.first() {
            return Ok(transaction.hash.clone());
        }
        if block_number == 0 {
            break;
        }
    }
    Err(format!("no transaction found near block {target_block}"))
}

async fn get_receipt<C: Client + Clone>(provider: &EthereumProvider<C>, transaction_hash: &str, missing_error: String) -> Result<u64, String> {
    provider
        .get_transaction_receipt(transaction_hash)
        .await
        .map_err(|error| error.to_string())?
        .map(|receipt| receipt.block_number)
        .ok_or(missing_error)
}

async fn get_receipt_at_age<C: Client + Clone>(provider: &EthereumProvider<C>, latest_block: u64, age: Duration, age_label: &str) -> Result<String, String> {
    let block_time = u64::from(provider.get_chain().block_time()).max(1);
    let block_depth = age.as_secs().saturating_mul(1_000).div_ceil(block_time).max(1);
    let unavailable = |error| format!("not available: {age_label} back: {error}");
    let transaction_hash = find_transaction_hash(provider, latest_block, block_depth).await.map_err(&unavailable)?;
    get_receipt(provider, &transaction_hash, "transaction receipt not found".to_string())
        .await
        .map(|block_number| format!("available at block {block_number}, {age_label} back"))
        .map_err(unavailable)
}
