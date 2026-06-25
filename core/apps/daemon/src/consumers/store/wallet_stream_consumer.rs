use std::error::Error;

use async_trait::async_trait;
use cacher::CacherClient;
use primitives::{StreamBalanceUpdate, StreamEvent, StreamTransactionsUpdate, StreamWalletUpdate, WalletId, device_stream_channel};
use storage::{Database, WalletsRepository};
use streamer::{WalletStreamEvent, WalletStreamPayload, consumer::MessageConsumer};

pub struct WalletStreamConsumer {
    pub database: Database,
    pub cacher_client: CacherClient,
}

fn stream_events(wallet_id: WalletId, event: WalletStreamEvent) -> Vec<StreamEvent> {
    match event {
        WalletStreamEvent::Transactions { transaction_ids, asset_ids } => std::iter::once(StreamEvent::Transactions(StreamTransactionsUpdate {
            wallet_id: wallet_id.clone(),
            transactions: transaction_ids,
        }))
        .chain(asset_ids.into_iter().map(|asset_id| {
            StreamEvent::Balances(StreamBalanceUpdate {
                wallet_id: wallet_id.clone(),
                asset_id,
            })
        }))
        .collect(),
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
        let wallet_id = wallet.wallet_id.0;
        let events = stream_events(wallet_id, payload.event);

        let mut count = 0;
        for device in &devices {
            let channel = device_stream_channel(&device.device_id);
            for event in &events {
                self.cacher_client.publish(&channel, event).await?;
                count += 1;
            }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use primitives::{AssetId, Chain, TransactionId};

    use super::*;

    #[test]
    fn test_stream_events_sends_transactions_before_balances() {
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

        assert_eq!(events.len(), 2);
        match &events[0] {
            StreamEvent::Transactions(update) => {
                assert_eq!(update.wallet_id, wallet_id);
                assert_eq!(update.transactions, vec![transaction_id]);
            }
            _ => panic!("expected transactions event"),
        }
        match &events[1] {
            StreamEvent::Balances(update) => {
                assert_eq!(update.wallet_id, wallet_id);
                assert_eq!(update.asset_id, asset_id);
            }
            _ => panic!("expected balances event"),
        }
    }
}
