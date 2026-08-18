use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use primitives::{StreamEvent, StreamTransactionsUpdate, StreamWalletUpdate, WalletId, device_stream_channel, unix_timestamp};
use storage::{Database, WalletsRepository};
use streamer::{WalletStreamEvent, WalletStreamPayload, consumer::MessageConsumer};

pub struct WalletStreamConsumer {
    pub database: Database,
    pub cacher_client: CacherClient,
    pub retention: Duration,
}

fn stream_events(wallet_id: WalletId, event: WalletStreamEvent) -> Vec<StreamEvent> {
    match event {
        WalletStreamEvent::Transactions { transaction_ids, asset_ids } => vec![StreamEvent::Transactions(StreamTransactionsUpdate {
            wallet_id,
            transactions: transaction_ids,
            asset_ids,
        })],
        WalletStreamEvent::FiatTransaction => vec![StreamEvent::FiatTransaction(StreamWalletUpdate { wallet_id })],
        WalletStreamEvent::Nft => vec![StreamEvent::Nft(StreamWalletUpdate { wallet_id })],
        WalletStreamEvent::Perpetual => vec![StreamEvent::Perpetual(StreamWalletUpdate { wallet_id })],
    }
}

#[async_trait]
impl MessageConsumer<WalletStreamPayload, usize> for WalletStreamConsumer {
    async fn should_process(&self, _payload: &WalletStreamPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: WalletStreamPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let wallet = self.database.wallets()?.get_wallet_by_id(payload.wallet_id)?;
        let devices = self.database.wallets()?.get_devices_by_wallet_id(payload.wallet_id)?;
        let events = stream_events(wallet.wallet_id.0, payload.event);
        let now = unix_timestamp();
        let expires_at = now.saturating_add(self.retention.as_secs()) as f64;

        for device in &devices {
            let channel = device_stream_channel(&device.device_id);
            let mut missed_events = Vec::new();
            for event in &events {
                let subscribers: usize = self.cacher_client.publish(&channel, event).await?;
                if subscribers == 0 {
                    missed_events.push((serde_json::to_string(event)?, expires_at));
                }
            }
            if missed_events.is_empty() {
                continue;
            }

            let cache_key = CacheKey::DeviceStreamEvents(&device.device_id, self.retention.as_secs());
            let expired_events = self
                .cacher_client
                .sorted_set_range_with_scores(&cache_key.key(), 0, -1)
                .await?
                .into_iter()
                .filter(|(_, score)| *score <= now as f64)
                .map(|(event, _)| event)
                .collect::<Vec<_>>();
            self.cacher_client.remove_from_sorted_set_cached(cache_key, &expired_events).await?;
            self.cacher_client
                .add_to_sorted_set_cached(CacheKey::DeviceStreamEvents(&device.device_id, self.retention.as_secs()), &missed_events)
                .await?;
        }
        Ok(devices.len() * events.len())
    }
}

#[cfg(test)]
mod tests {
    use primitives::{AssetId, Chain, TransactionId};

    use super::*;

    #[test]
    fn test_stream_events_sends_transaction_with_affected_assets() {
        let wallet_id = WalletId::Multicoin("wallet".to_string());
        let transaction_id = TransactionId::new(Chain::Ethereum, "0x123".to_string());
        let asset_id = AssetId::from_chain(Chain::Ethereum);

        let events = stream_events(
            wallet_id.clone(),
            WalletStreamEvent::Transactions {
                transaction_ids: vec![transaction_id.clone()],
                asset_ids: vec![asset_id.clone()],
            },
        );

        assert_eq!(events.len(), 1);
        match &events[0] {
            StreamEvent::Transactions(update) => {
                assert_eq!(update.wallet_id, wallet_id);
                assert_eq!(update.transactions, vec![transaction_id]);
                assert_eq!(update.asset_ids, vec![asset_id]);
            }
            _ => panic!("expected transaction event"),
        }
    }
}
