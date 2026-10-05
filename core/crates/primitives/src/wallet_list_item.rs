use model_derive::Model;
use serde::{Deserialize, Serialize};

use crate::{Wallet, WalletId};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[model(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct WalletListItem {
    pub id: WalletId,
    pub name: String,
    pub index: i32,
    pub is_pinned: bool,
    pub image_url: Option<String>,
}

impl From<&Wallet> for WalletListItem {
    fn from(wallet: &Wallet) -> Self {
        Self {
            id: wallet.id.clone(),
            name: wallet.name.clone(),
            index: wallet.index,
            is_pinned: wallet.is_pinned,
            image_url: wallet.image_url.clone(),
        }
    }
}
