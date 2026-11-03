use gem_client::Target;

#[derive(Clone, Debug)]
pub enum TopperTarget {
    Assets,
    Countries,
    PaymentMethods,
    Simulations,
}

impl Target for TopperTarget {
    fn path(&self) -> String {
        match self {
            Self::Assets => "/assets/crypto-onramp".to_string(),
            Self::Countries => "/countries?flow=crypto_onramp".to_string(),
            Self::PaymentMethods => "/payment-methods/crypto-onramp".to_string(),
            Self::Simulations => "/simulations".to_string(),
        }
    }
}
