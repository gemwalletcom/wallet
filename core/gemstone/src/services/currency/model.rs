use primitives::Currency;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCurrencyRow {
    pub currency: Currency,
    pub title: String,
    pub is_selected: bool,
}
