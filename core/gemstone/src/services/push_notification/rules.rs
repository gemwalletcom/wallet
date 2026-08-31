use primitives::{PushNotificationAsset, PushNotificationSwapAsset, PushNotificationTransaction, PushNotificationTypes, PushNotificationWalletAsset};
use serde::de::DeserializeOwned;

use super::model::GemPushNotification;

pub fn notification(notification_type: &str, data: Option<&str>) -> Option<GemPushNotification> {
    match notification_type.parse::<PushNotificationTypes>().ok()? {
        PushNotificationTypes::Test => Some(GemPushNotification::Test),
        PushNotificationTypes::Support => Some(GemPushNotification::Support),
        PushNotificationTypes::Rewards => Some(GemPushNotification::Rewards),
        PushNotificationTypes::Transaction => payload::<PushNotificationTransaction>(data).map(|payload| GemPushNotification::Transaction {
            wallet_id: payload.wallet_id,
            asset_id: payload.asset_id,
            transaction: payload.transaction,
        }),
        PushNotificationTypes::Asset => payload::<PushNotificationAsset>(data).map(|payload| GemPushNotification::Asset { asset_id: payload.asset_id }),
        PushNotificationTypes::PriceAlert => payload::<PushNotificationAsset>(data).map(|payload| GemPushNotification::PriceAlert { asset_id: payload.asset_id }),
        PushNotificationTypes::BuyAsset => payload::<PushNotificationAsset>(data).map(|payload| GemPushNotification::BuyAsset { asset_id: payload.asset_id }),
        PushNotificationTypes::SwapAsset => payload::<PushNotificationSwapAsset>(data).map(|payload| GemPushNotification::SwapAsset {
            from_asset_id: payload.from_asset_id,
            to_asset_id: payload.to_asset_id,
        }),
        PushNotificationTypes::FiatTransaction => payload::<PushNotificationWalletAsset>(data).map(|payload| GemPushNotification::FiatTransaction {
            wallet_id: payload.wallet_id,
            asset_id: payload.asset_id,
        }),
        PushNotificationTypes::Stake => payload::<PushNotificationWalletAsset>(data).map(|payload| GemPushNotification::Stake {
            wallet_id: payload.wallet_id,
            asset_id: payload.asset_id,
        }),
    }
}

fn payload<T: DeserializeOwned>(data: Option<&str>) -> Option<T> {
    serde_json::from_str(data?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, PushNotification, PushNotificationReward, Transaction, WalletId};

    fn envelope(notification: &PushNotification) -> (String, Option<String>) {
        (notification.notification_type.as_ref().to_string(), notification.data.as_ref().map(|data| data.to_string()))
    }

    fn parse(envelope: &PushNotification) -> Option<GemPushNotification> {
        let (notification_type, data) = self::envelope(envelope);
        notification(&notification_type, data.as_deref())
    }

    #[test]
    fn test_notification_maps_backend_envelopes() {
        let bitcoin = AssetId::from_chain(Chain::Bitcoin);
        let solana = AssetId::from_chain(Chain::Solana);
        let wallet_id = WalletId::Multicoin("0x123".to_string());

        assert_eq!(
            parse(&PushNotification::new_buy_asset(bitcoin.clone())),
            Some(GemPushNotification::BuyAsset { asset_id: bitcoin.clone() })
        );
        assert_eq!(
            parse(&PushNotification::new_fiat_transaction(wallet_id.clone(), bitcoin.clone())),
            Some(GemPushNotification::FiatTransaction {
                wallet_id: wallet_id.clone(),
                asset_id: bitcoin.clone(),
            })
        );
        assert_eq!(
            parse(&PushNotification::new_stake(wallet_id.clone(), solana.clone())),
            Some(GemPushNotification::Stake {
                wallet_id: wallet_id.clone(),
                asset_id: solana.clone(),
            })
        );

        let asset_payload = serde_json::to_string(&PushNotificationAsset { asset_id: bitcoin.clone() }).unwrap();
        assert_eq!(
            notification(PushNotificationTypes::Asset.as_ref(), Some(&asset_payload)),
            Some(GemPushNotification::Asset { asset_id: bitcoin.clone() })
        );
        assert_eq!(
            notification(PushNotificationTypes::PriceAlert.as_ref(), Some(&asset_payload)),
            Some(GemPushNotification::PriceAlert { asset_id: bitcoin.clone() })
        );

        let swap_payload = serde_json::to_string(&PushNotificationSwapAsset {
            from_asset_id: bitcoin.clone(),
            to_asset_id: solana.clone(),
        })
        .unwrap();
        assert_eq!(
            notification(PushNotificationTypes::SwapAsset.as_ref(), Some(&swap_payload)),
            Some(GemPushNotification::SwapAsset {
                from_asset_id: bitcoin.clone(),
                to_asset_id: solana.clone(),
            })
        );

        let transaction = Transaction::mock();
        let transaction_payload = serde_json::to_string(&PushNotificationTransaction {
            wallet_id: wallet_id.clone(),
            asset_id: transaction.asset_id.clone(),
            transaction_id: transaction.id.to_string(),
            transaction: transaction.clone(),
        })
        .unwrap();
        assert_eq!(
            notification(PushNotificationTypes::Transaction.as_ref(), Some(&transaction_payload)),
            Some(GemPushNotification::Transaction {
                wallet_id,
                asset_id: transaction.asset_id.clone(),
                transaction,
            })
        );
    }

    #[test]
    fn test_notification_keeps_price_alert_apart_from_asset() {
        let asset_id = AssetId::from_chain(Chain::Ethereum);
        let data = serde_json::to_string(&PushNotificationAsset { asset_id: asset_id.clone() }).unwrap();

        let price_alert = notification(PushNotificationTypes::PriceAlert.as_ref(), Some(&data));
        let asset = notification(PushNotificationTypes::Asset.as_ref(), Some(&data));

        assert_eq!(price_alert, Some(GemPushNotification::PriceAlert { asset_id: asset_id.clone() }));
        assert_eq!(asset, Some(GemPushNotification::Asset { asset_id }));
        assert_ne!(price_alert, asset);
    }

    #[test]
    fn test_notification_without_payload_keeps_destination_only_types() {
        for notification_type in [PushNotificationTypes::Rewards, PushNotificationTypes::Support, PushNotificationTypes::Test] {
            assert!(notification(notification_type.as_ref(), None).is_some(), "type: {}", notification_type.as_ref());
            assert!(notification(notification_type.as_ref(), Some("not json")).is_some(), "type: {}", notification_type.as_ref());
        }

        let reward = serde_json::to_string(&PushNotificationReward { wallet_id: "1".to_string() }).unwrap();
        assert_eq!(notification(PushNotificationTypes::Rewards.as_ref(), Some(&reward)), Some(GemPushNotification::Rewards));
        assert_eq!(notification(PushNotificationTypes::Test.as_ref(), None), Some(GemPushNotification::Test));
    }

    #[test]
    fn test_notification_rejects_unusable_envelopes() {
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let data = serde_json::to_string(&PushNotificationAsset { asset_id }).unwrap();

        assert_eq!(notification("unknownType", Some(&data)), None);
        assert_eq!(notification("", Some(&data)), None);
        assert_eq!(notification(PushNotificationTypes::Asset.as_ref(), None), None);
        assert_eq!(notification(PushNotificationTypes::Asset.as_ref(), Some("")), None);
        assert_eq!(notification(PushNotificationTypes::Asset.as_ref(), Some(r#"{"assetId":"unknown"}"#)), None);
        assert_eq!(notification(PushNotificationTypes::Transaction.as_ref(), Some(&data)), None);
        assert_eq!(notification(PushNotificationTypes::Stake.as_ref(), Some(&data)), None);
    }
}
