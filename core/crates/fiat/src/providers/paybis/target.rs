use gem_client::Target;

#[derive(Clone, Debug)]
pub enum PaybisTarget {
    Quote,
    Request,
    CurrencyPairs { flow: String },
    SellCurrencyPairs,
}

impl Target for PaybisTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote => "/v2/quote".to_string(),
            Self::Request => "/v3/request".to_string(),
            Self::CurrencyPairs { flow } => format!("/v2/currency/pairs/{flow}"),
            Self::SellCurrencyPairs => "/v2/currency/pairs/sell-crypto".to_string(),
        }
    }
}
