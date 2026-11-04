use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

use primitives::{AssetId, Platform, Transaction, Version, WalletId};

#[derive(Debug, Serialize, Deserialize, Clone, AsRefStr, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum PushNotificationTypes {
    Test,
    Transaction,
    Asset,
    PriceAlert,
    BuyAsset,
    SwapAsset,
    Support,
    Rewards,
    Stake,
    FiatTransaction,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PushNotification {
    #[serde(rename = "type")]
    pub notification_type: PushNotificationTypes,
    pub data: Option<serde_json::Value>,
}

impl PushNotification {
    pub fn new_buy_asset(asset_id: AssetId) -> Self {
        Self {
            notification_type: PushNotificationTypes::BuyAsset,
            data: serde_json::to_value(PushNotificationAsset { asset_id }).ok(),
        }
    }

    pub fn new_fiat_transaction(wallet_id: WalletId, asset_id: AssetId) -> Self {
        Self {
            notification_type: PushNotificationTypes::FiatTransaction,
            data: serde_json::to_value(PushNotificationWalletAsset { wallet_id, asset_id }).ok(),
        }
    }

    pub fn new_transaction(wallet_id: WalletId, transaction: &Transaction, platform: Platform, app_version: &str) -> Self {
        let payload = PushNotificationTransaction {
            wallet_id,
            asset_id: transaction.asset_id.clone(),
            transaction_id: transaction.id.to_string(),
            transaction: carries_transaction(platform, app_version).then(|| transaction.clone().with_data(None)),
        };
        Self {
            notification_type: PushNotificationTypes::Transaction,
            data: serde_json::to_value(payload).ok(),
        }
    }

    pub fn new_stake(wallet_id: WalletId, asset_id: AssetId) -> Self {
        Self {
            notification_type: PushNotificationTypes::Stake,
            data: serde_json::to_value(PushNotificationWalletAsset { wallet_id, asset_id }).ok(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationTransaction {
    pub wallet_id: WalletId,
    pub asset_id: AssetId,
    pub transaction_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<Transaction>,
}

fn carries_transaction(platform: Platform, app_version: &str) -> bool {
    let reads_transaction_id = match platform {
        Platform::IOS => Version::new(2, 114, 74),
        Platform::Android => Version::new(2, 114, 73),
    };
    !app_version.parse::<Version>().is_ok_and(|version| version >= reads_transaction_id)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationAsset {
    pub asset_id: AssetId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationSwapAsset {
    pub from_asset_id: AssetId,
    pub to_asset_id: AssetId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationSupport {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationReward {
    pub wallet_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationWalletAsset {
    pub wallet_id: WalletId,
    pub asset_id: AssetId,
}

#[cfg(test)]
impl PushNotification {
    pub(crate) fn mock() -> Self {
        Self {
            notification_type: PushNotificationTypes::Test,
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_transaction_push_leaves_out_the_transaction_for_apps_that_read_the_id() {
        let transaction = Transaction::mock();
        let payload = |platform: Platform, app_version: &str| {
            serde_json::from_value::<PushNotificationTransaction>(PushNotification::new_transaction(WalletId::Multicoin("0x1".to_string()), &transaction, platform, app_version).data.unwrap()).unwrap()
        };

        assert_eq!(payload(Platform::Android, "2.114.72").transaction, Some(transaction.clone()));
        assert_eq!(payload(Platform::Android, "2.114.73").transaction, None);
        assert_eq!(payload(Platform::IOS, "2.114.73").transaction, Some(transaction.clone()));
        assert_eq!(payload(Platform::IOS, "2.114.74").transaction, None);
        assert_eq!(payload(Platform::IOS, "2.115").transaction, None);
        assert_eq!(payload(Platform::IOS, "unknown").transaction, Some(transaction.clone()), "an unreadable version keeps the payload old apps need");
        assert_eq!(payload(Platform::IOS, "2.114.74").transaction_id, transaction.id.to_string());
        assert_eq!(payload(Platform::IOS, "2.114.74").asset_id, transaction.asset_id);
    }

    #[test]
    fn test_new_transaction_leaves_out_call_data() {
        let transaction = Transaction::mock().with_data(Some("0xa9059cbb".to_string()));

        let data = PushNotification::new_transaction(WalletId::Multicoin("0x1".to_string()), &transaction, Platform::Android, "2.114.72").data.unwrap();

        assert_eq!(serde_json::from_value::<PushNotificationTransaction>(data).unwrap().transaction, Some(transaction.with_data(None)));
    }
}
