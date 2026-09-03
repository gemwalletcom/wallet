use gem_client::{ReqwestClient, reqwest_client};
use settings::Name;

use crate::providers::{
    alldomains::AllDomainsProvider, aptos::AptosProvider, basenames::BasenamesProvider, did::DidProvider, ens::EnsProvider, eths::EthsProvider, hyperliquid::HyperliquidProvider,
    icns::IcnsProvider, injective::InjectiveProvider, lens::LensProvider, near::NearProvider, sns::SnsProvider, spaceid::SpaceIdProvider, suins::SuinsProvider, ton::TonProvider,
    ud::UdProvider,
};
use crate::resolver::NameResolver;

pub struct NameProviderFactory;

impl NameProviderFactory {
    pub fn new_providers(config: Name) -> Vec<Box<dyn NameResolver>> {
        let client = ReqwestClient::new(String::new(), reqwest_client());
        let client_with_url = |url: String| client.clone().with_base_url(url);

        vec![
            Box::new(EnsProvider::new(client_with_url(config.ens.url))),
            Box::new(UdProvider::new(client_with_url(config.ud.url))),
            Box::new(SnsProvider::new(client_with_url(config.sns.url))),
            Box::new(TonProvider::new(client_with_url(config.ton.url))),
            Box::new(EthsProvider::new(client_with_url(config.eths.url))),
            Box::new(SpaceIdProvider::new(client_with_url(config.spaceid.url))),
            Box::new(DidProvider::new(client_with_url(config.did.url))),
            Box::new(SuinsProvider::new(config.suins.url)),
            Box::new(AptosProvider::new(client_with_url(config.aptos.url))),
            Box::new(InjectiveProvider::new(client_with_url(config.injective.url))),
            Box::new(IcnsProvider::new(client_with_url(config.icns.url))),
            Box::new(LensProvider::new(client_with_url(config.lens.url))),
            Box::new(BasenamesProvider::new(client_with_url(config.base.url))),
            Box::new(HyperliquidProvider::new(client_with_url(config.hyperliquid.url))),
            Box::new(AllDomainsProvider::new(client_with_url(config.alldomains.url))),
            Box::new(NearProvider::new(client_with_url(config.near.url))),
        ]
    }
}
