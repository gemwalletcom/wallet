use gem_client::{Client, ClientError, ClientExt};
use primitives::{Chain, StakeValidator};

use crate::static_target::GemStaticApiTarget;

#[derive(Debug, Clone)]
pub struct GemStaticApiClient<C: Client> {
    client: C,
}

impl<C: Client> GemStaticApiClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_validators(&self, chain: Chain) -> Result<Vec<StakeValidator>, ClientError> {
        self.client.get(&GemStaticApiTarget::GetValidators(chain).path()).await
    }
}
