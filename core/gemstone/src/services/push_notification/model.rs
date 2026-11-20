use primitives::{AssetId, TransactionId, WalletId};

#[derive(Debug, Clone, PartialEq)]
pub enum GemPushNotification {
    Transaction { wallet_id: WalletId, asset_id: AssetId, transaction_id: TransactionId },
    Asset { asset_id: AssetId },
    PriceAlert { asset_id: AssetId },
    BuyAsset { asset_id: AssetId },
    SwapAsset { from_asset_id: AssetId, to_asset_id: AssetId },
    FiatTransaction { wallet_id: WalletId, asset_id: AssetId },
    Stake { wallet_id: WalletId, asset_id: AssetId },
    Support,
    Rewards,
    Test,
}
