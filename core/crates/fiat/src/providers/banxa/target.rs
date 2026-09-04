use gem_client::{Target, build_path_with_query};

use super::models::BuyQuoteQuery;

const BUY_ORDER_TYPE: &str = "buy";

#[derive(Clone, Debug)]
pub enum BanxaTarget {
    Assets { partner: String },
    Order { partner: String, order_id: String },
    BuyQuote { partner: String, query: BuyQuoteQuery },
    Countries { partner: String },
    FiatCurrencies { partner: String },
    CreateBuyOrder { partner: String },
}

impl Target for BanxaTarget {
    fn path(&self) -> String {
        match self {
            Self::Assets { partner } => format!("/{partner}/v2/crypto/{BUY_ORDER_TYPE}"),
            Self::Order { partner, order_id } => format!("/{partner}/v2/orders/{order_id}"),
            Self::BuyQuote { partner, query } => build_path_with_query(&format!("/{partner}/v2/quotes/buy"), query),
            Self::Countries { partner } => format!("/{partner}/v2/countries"),
            Self::FiatCurrencies { partner } => format!("/{partner}/v2/fiats/{BUY_ORDER_TYPE}"),
            Self::CreateBuyOrder { partner } => format!("/{partner}/v2/buy"),
        }
    }
}
