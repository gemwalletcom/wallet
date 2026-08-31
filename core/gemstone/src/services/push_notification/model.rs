use primitives::{AssetId, Transaction, WalletId};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemPushNotification {
    Transaction {
        wallet_id: WalletId,
        asset_id: AssetId,
        transaction: Transaction,
    },
    Asset {
        asset_id: AssetId,
    },
    PriceAlert {
        asset_id: AssetId,
    },
    BuyAsset {
        asset_id: AssetId,
    },
    SwapAsset {
        from_asset_id: AssetId,
        to_asset_id: AssetId,
    },
    FiatTransaction {
        wallet_id: WalletId,
        asset_id: AssetId,
    },
    Stake {
        wallet_id: WalletId,
        asset_id: AssetId,
    },
    Support,
    Rewards,
    Test,
}
