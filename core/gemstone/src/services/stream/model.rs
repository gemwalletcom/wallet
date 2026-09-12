use primitives::{AssetId, TransactionId, WalletId};

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemStreamEvent {
    Prices {
        prices: u32,
        rates: u32,
    },
    Balances {
        wallet_id: WalletId,
        asset_ids: Vec<AssetId>,
    },
    Transactions {
        wallet_id: WalletId,
        transaction_ids: Vec<TransactionId>,
        asset_ids: Vec<AssetId>,
    },
    PriceAlerts {
        asset_ids: Vec<AssetId>,
    },
    Nft {
        wallet_id: WalletId,
    },
    Perpetual {
        wallet_id: WalletId,
    },
    Notification {
        wallet_id: WalletId,
    },
    FiatTransaction {
        wallet_id: WalletId,
    },
    SupportMessage {
        message_id: String,
        images: u32,
    },
    SupportTyping {
        is_typing: bool,
    },
}
