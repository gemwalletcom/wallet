#[cfg(test)]
mod tests {
    use gem_client::{ReqwestClient, reqwest_client};
    use name_resolver::providers::{
        alldomains::AllDomainsProvider, aptos::AptosProvider, basenames::BasenamesProvider, ens::EnsProvider, hyperliquid::HyperliquidProvider, icns::IcnsProvider, injective::InjectiveProvider, lens::LensProvider, near::NearProvider,
        sns::SnsProvider, suins::SuinsProvider,
    };
    use name_resolver::{NameClient, NameConfig, NameQuery, NameResolver};
    use primitives::{Chain, NameProvider, node_config::get_nodes_for_chain};
    use settings::testkit::get_test_settings;

    #[tokio::test]
    async fn test_resolve_ens() {
        let provider = EnsProvider::new(ReqwestClient::new(get_nodes_for_chain(Chain::Ethereum)[0].url.clone(), reqwest_client()));
        let address = provider.resolve(&NameQuery::new("vitalik.eth"), Chain::Ethereum).await.unwrap().unwrap();
        assert_eq!(address, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
    }

    #[tokio::test]
    async fn test_resolve_ens_imported_name() {
        let client = NameClient::new(
            vec![Box::new(EnsProvider::new(ReqwestClient::new(get_nodes_for_chain(Chain::Ethereum)[0].url.clone(), reqwest_client())))],
            NameConfig { max_name_length: 20 },
        );
        let record = client.resolve("farcaster.xyz", Chain::Ethereum).await.unwrap().unwrap();
        assert_eq!(record.address, "0xF12E89805E10d96c0CDf22da88aED361eD9329cA");
    }

    #[tokio::test]
    async fn test_resolve_basenames() {
        let provider = BasenamesProvider::new(ReqwestClient::new(get_nodes_for_chain(Chain::Base)[0].url.clone(), reqwest_client()));
        let address = provider.resolve(&NameQuery::new("h3rman.base.eth"), Chain::Base).await.unwrap().unwrap();
        assert_eq!(address, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
    }

    #[tokio::test]
    async fn test_resolve_sns() {
        let client = NameClient::new(vec![Box::new(SnsProvider::new(ReqwestClient::new(get_test_settings().name.sns.url, reqwest_client())))], NameConfig { max_name_length: 20 });
        for name in ["bonfida.sol", "bonfida.sns"] {
            let record = client.resolve(name, Chain::Solana).await.unwrap().unwrap();
            assert_eq!(record.name, name);
            assert_eq!(record.provider, NameProvider::Sns);
            assert_eq!(record.chain, Chain::Solana);
            assert_eq!(record.address.is_empty(), false);
        }
    }

    #[tokio::test]
    async fn test_resolve_injective() {
        let provider = InjectiveProvider::new(ReqwestClient::new(get_nodes_for_chain(Chain::Injective)[0].url.clone(), reqwest_client()));
        let address = provider.resolve(&NameQuery::new("test.inj"), Chain::Injective).await.unwrap().unwrap();
        assert_eq!(address, "inj14apqz6u2nprsly3j0mqa6jwpxnmnphq3pp0q9g");
    }

    #[tokio::test]
    async fn test_resolve_icns() {
        let client = NameClient::new(vec![Box::new(IcnsProvider::new(ReqwestClient::new(get_test_settings().name.icns.url, reqwest_client())))], NameConfig { max_name_length: 20 });
        let record = client.resolve("dogemos.osmo", Chain::Osmosis).await.unwrap().unwrap();
        assert_eq!(record.address, "osmo1z98eg2ztdp2glyla62629nrlvczg8s7f8sgpm5");
    }

    #[tokio::test]
    async fn test_resolve_lens() {
        let provider = LensProvider::new(ReqwestClient::new(get_test_settings().name.lens.url, reqwest_client()));
        let address = provider.resolve(&NameQuery::new("stani.lens"), Chain::Ethereum).await.unwrap().unwrap();
        assert_eq!(address, "0xAd2c0BEAdE60fb9f7ec5C87bDE8e4c126145F6E7");
    }

    #[tokio::test]
    async fn test_resolve_suins() {
        let provider = SuinsProvider::new(get_nodes_for_chain(Chain::Sui)[0].url.clone());
        let address = provider.resolve(&NameQuery::new("alpha.sui"), Chain::Sui).await.unwrap().unwrap();
        assert_eq!(address, "0x54e5c2a6f1276ac2ff623ac54e53e5a61a576906b3ec42fac8fe8bf5615d0957");
    }

    #[tokio::test]
    async fn test_resolve_aptos() {
        let provider = AptosProvider::new(ReqwestClient::new(get_test_settings().name.aptos.url, reqwest_client()));
        let address = provider.resolve(&NameQuery::new("petra.apt"), Chain::Aptos).await.unwrap().unwrap();
        assert_eq!(address, "0xfe2ffdb3a74307f7314a1c8ab3762b6b5869a3c1278cdd5d230249453e15a1db");
    }

    #[tokio::test]
    async fn test_resolve_hyperliquid() {
        let provider = HyperliquidProvider::new(ReqwestClient::new(get_test_settings().name.hyperliquid.url, reqwest_client()));
        let query = NameQuery::new("TESTOOOR.HL");

        assert_eq!(provider.resolve(&query, Chain::Ethereum).await.unwrap().unwrap(), "0xb43f5153B1c867BF78ACB3C35aa9b8ae366415c5");
        assert_eq!(provider.resolve(&query, Chain::Hyperliquid).await.unwrap().unwrap(), "0xF26F5551E96aE5162509B25925fFfa7F07B2D652");
        assert_eq!(provider.resolve(&query, Chain::Solana).await.unwrap().unwrap(), "CKAvaYmwqCbg8nZCUCNj6Cvr11HauALtNoGT7WirPoAp");
    }

    #[tokio::test]
    async fn test_resolve_alldomains() {
        let provider = AllDomainsProvider::new(ReqwestClient::new(get_nodes_for_chain(Chain::Solana)[0].url.clone(), reqwest_client()));
        let address = provider.resolve(&NameQuery::new("miester.poor"), Chain::Solana).await.unwrap().unwrap();
        assert_eq!(address, "2EGGxj2qbNAJNgLCPKca8sxZYetyTjnoRspTPjzN2D67");
    }

    #[tokio::test]
    async fn test_resolve_near() {
        let provider = NearProvider::new(ReqwestClient::new(get_nodes_for_chain(Chain::Near)[0].url.clone(), reqwest_client()));
        let address = provider.resolve(&NameQuery::new("wrap.near"), Chain::Near).await.unwrap().unwrap();
        assert_eq!(address, "wrap.near");
    }
}
