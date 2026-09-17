use primitives::Chain;

use crate::formatted_number::GemFormattedNumber;
use crate::models::copy::GemCopy;
use crate::services::error::GemServiceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemListSectionTitle {
    None,
    Balances,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemListRowTitle {
    Name,
    Network,
    Address,
    Available,
    Stake,
    Earn,
    PendingUnconfirmed,
    Reserved,
    Error,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemListRow {
    Text { title: GemListRowTitle, value: String },
    Amount { title: GemListRowTitle, amount: GemFormattedNumber },
    Icon { chain: Chain },
    Address { address: String, copy: GemCopy },
    Explorer { name: String, url: String },
    Loading,
    Error { error: GemServiceError },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemListSection {
    pub title: GemListSectionTitle,
    pub rows: Vec<GemListRow>,
}
