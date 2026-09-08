use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, InAppNotification, SupportStreamEvent, TransactionId, WalletId, WebSocketPricePayload};

pub const DEVICE_STREAM_CHANNEL_PREFIX: &str = "stream:device:";

pub fn device_stream_channel(device_id: &str) -> String {
    format!("{DEVICE_STREAM_CHANNEL_PREFIX}{device_id}")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
#[allow(clippy::large_enum_variant)]
pub enum StreamEvent {
    Prices(WebSocketPricePayload),
    Balances(StreamBalanceUpdate),
    Transactions(StreamTransactionsUpdate),
    PriceAlerts(StreamPriceAlertUpdate),
    Nft(StreamWalletUpdate),
    Perpetual(StreamWalletUpdate),
    InAppNotification(StreamNotificationUpdate),
    FiatTransaction(StreamWalletUpdate),
    Support(SupportStreamEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamMessagePrices {
    pub assets: Vec<AssetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub enum StreamMessage {
    GetPrices(StreamMessagePrices),
    SubscribePrices(StreamMessagePrices),
    UnsubscribePrices(StreamMessagePrices),
    AddPrices(StreamMessagePrices),
    SubscribeRealtimePrices(StreamMessagePrices),
    UnsubscribeRealtimePrices(StreamMessagePrices),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamBalanceUpdate {
    pub wallet_id: WalletId,
    pub asset_ids: Vec<AssetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamTransactionsUpdate {
    pub wallet_id: WalletId,
    pub transactions: Vec<TransactionId>,
    pub asset_ids: Vec<AssetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamPriceAlertUpdate {
    pub assets: Vec<AssetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamWalletUpdate {
    pub wallet_id: WalletId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamNotificationUpdate {
    pub wallet_id: WalletId,
    pub notification: InAppNotification,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chain;

    #[test]
    fn test_balance_update_requires_asset_ids_not_legacy_asset_id() {
        for payload in [
            r#"{"walletId":"multicoin_0x1","assetIds":["ethereum","solana"]}"#,
            r#"{"walletId":"multicoin_0x1","assetId":"ethereum","assetIds":["ethereum","solana"]}"#,
        ] {
            let update: StreamBalanceUpdate = serde_json::from_str(payload).unwrap();

            assert_eq!(update.wallet_id, WalletId::Multicoin("0x1".into()));
            assert_eq!(update.asset_ids, vec![AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Solana)]);
        }

        let legacy_only = r#"{"walletId":"multicoin_0x1","assetId":"ethereum"}"#;
        assert!(serde_json::from_str::<StreamBalanceUpdate>(legacy_only).is_err());
    }
}
