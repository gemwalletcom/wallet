use primitives::Currency;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCurrencyRow {
    pub currency: Currency,
    pub title: String,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemCurrencySectionKind {
    Recommended,
    All,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCurrencySection {
    pub kind: GemCurrencySectionKind,
    pub rows: Vec<GemCurrencyRow>,
}
