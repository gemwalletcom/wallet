use primitives::Currency;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCurrencyRow {
    pub currency: Currency,
    pub flag: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCurrencies {
    pub selected: GemCurrencyRow,
    pub recommended: Vec<GemCurrencyRow>,
    pub other: Vec<GemCurrencyRow>,
}
