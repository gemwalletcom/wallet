use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use cacher::DeviceStreamCacher;
use primitives::{StreamEvent, StreamTransactionsUpdate, StreamWalletUpdate, WalletId, unix_timestamp};
use storage::{Database, DatabaseError, WalletsRepository};
use streamer::{WalletStreamEvent, WalletStreamPayload, consumer::MessageConsumer};

pub struct WalletStreamConsumer {
    pub database: Database,
    pub device_stream: Arc<dyn DeviceStreamCacher>,
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
        WalletStreamEvent::WalletConfiguration => vec![StreamEvent::WalletConfiguration(StreamWalletUpdate { wallet_id })],
    }
}

#[async_trait]
impl MessageConsumer<WalletStreamPayload, usize> for WalletStreamConsumer {
    async fn should_consume(&self, _payload: &WalletStreamPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn consume(&self, payload: WalletStreamPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let wallet_row_id = payload.wallet_id;
        let (wallet, devices) = self
            .database
            .run(move |client| -> Result<_, DatabaseError> { Ok((client.get_wallet_by_id(wallet_row_id)?, client.get_devices_by_wallet_id(wallet_row_id)?)) })
            .await?;
        let events = stream_events(wallet.wallet_id, payload.event);
        let expires_at = unix_timestamp().saturating_add(self.retention.as_secs()) as f64;

        for device in &devices {
            let mut missed_events = Vec::new();
            for event in &events {
                let subscribers = self.device_stream.publish_event(&device.id, event).await?;
                if subscribers == 0 {
                    missed_events.push((serde_json::to_string(event)?, expires_at));
                }
            }
            self.device_stream.add_events(&device.id, self.retention, &missed_events).await?;
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
