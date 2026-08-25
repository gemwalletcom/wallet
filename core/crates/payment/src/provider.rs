use primitives::{Chain, ChainAddress};

use crate::{PaymentError, PaymentTransaction};

pub(crate) trait PaymentProvider {
    fn supported_chains(&self) -> &'static [Chain];

    async fn load(&self, addresses: &[ChainAddress]) -> Result<PaymentTransaction, PaymentError>;
}
