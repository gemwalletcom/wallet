use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};
use primitives::FiatProviderName;

use super::model::{TopperAssetsResponse, TopperCountriesResponse, TopperOnRamp, TopperPaymentMethodsResponse, TopperSimulation, TopperSimulationRequest, TopperSimulationResponse};
use super::target::TopperTarget;
use super::token::TopperBootstrapToken;
use crate::provider::generate_quote_id;

pub struct TopperClient {
    client: ReqwestClient,
    pub(super) token: TopperBootstrapToken,
    pub(super) widget_url: String,
    pub(super) webhook_key: String,
}

impl TopperClient {
    pub const NAME: FiatProviderName = FiatProviderName::Topper;

    pub fn new(client: ReqwestClient, widget_url: String, widget_id: String, key_id: String, secret: String, webhook_key: String) -> Self {
        Self {
            client,
            token: TopperBootstrapToken::new(widget_id, key_id, secret),
            widget_url,
            webhook_key,
        }
    }

    pub async fn get_onramp_assets(&self) -> Result<TopperAssetsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TopperTarget::Assets).await?)
    }

    pub async fn get_onramp_countries(&self) -> Result<TopperCountriesResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TopperTarget::Countries).await?)
    }

    pub async fn get_onramp_payment_methods(&self) -> Result<TopperPaymentMethodsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TopperTarget::PaymentMethods).await?)
    }

    pub async fn simulate(&self, flow: TopperOnRamp) -> Result<TopperSimulation, Box<dyn Error + Send + Sync>> {
        let request = TopperSimulationRequest {
            bootstrap_token: self.token.sign(&generate_quote_id(), flow)?,
        };
        let response: TopperSimulationResponse = self.client.post(TopperTarget::Simulations, &request).await?;
        Ok(response.simulation)
    }
}
