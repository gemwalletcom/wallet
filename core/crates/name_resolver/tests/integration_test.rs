#[cfg(test)]
mod tests {
    use std::env;

    use name_resolver::{
        alldomains::AllDomainsClient,
        aptos::AptosClient,
        base::Basenames,
        client::{NameClient, NameConfig},
        ens::ENSClient,
        hyperliquid::Hyperliquid,
        icns::IcnsClient,
        injective::InjectiveNameClient,
        lens::LensClient,
        model::NameQuery,
        near::NearNameClient,
        suins::SuinsClient,
    };
    use primitives::{Chain, node_config::get_nodes_for_chain};
    use settings::Settings;

    #[tokio::test]
    async fn test_resolver_eth() {
        // this test is ignored from UT cause it connects to the real network
        let nodes = get_nodes_for_chain(Chain::Ethereum);
        let client = ENSClient::new(nodes[0].url.clone());
        let address = client.resolve(&NameQuery::new("vitalik.eth"), Chain::Ethereum).await;
        assert_eq!(address.unwrap(), "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
    }

    #[tokio::test]
    async fn test_resolver_ens_imported_name() {
        let nodes = get_nodes_for_chain(Chain::Ethereum);
        let client = name_resolver::client::Client::new(vec![Box::new(ENSClient::new(nodes[0].url.clone()))], NameConfig { max_name_length: 20 });
        let address = client.resolve("farcaster.xyz", Chain::Ethereum).await.unwrap().address;
        assert_eq!(address, "0xF12E89805E10d96c0CDf22da88aED361eD9329cA");
    }

    #[tokio::test]
    async fn test_resolve_basenames() {
        let nodes = get_nodes_for_chain(Chain::Base);
        let client = Basenames::new(nodes[0].url.clone());
        let address = client.resolve(&NameQuery::new("h3rman.base.eth"), Chain::Base).await.unwrap();
        assert_eq!(address.to_lowercase(), "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_lowercase())
    }

    #[tokio::test]
    async fn test_resolve_injective() {
        let nodes = get_nodes_for_chain(Chain::Injective);
        let client = InjectiveNameClient::new(nodes[0].url.clone());
        let address_result = client.resolve(&NameQuery::new("test.inj"), Chain::Injective).await;
        assert_eq!(address_result.unwrap(), "inj14apqz6u2nprsly3j0mqa6jwpxnmnphq3pp0q9g");
    }

    #[tokio::test]
    async fn test_resolve_icns() {
        let current_dir = env::current_dir().unwrap();
        let path = current_dir.join("../../Settings.yaml");
        let settings = Settings::new_setting_path(path).unwrap();
        let client = name_resolver::client::Client::new(vec![Box::new(IcnsClient::new(settings.name.icns.url))], NameConfig { max_name_length: 20 });
        let record = client.resolve("dogemos.osmo", Chain::Osmosis).await.unwrap();
        assert_eq!(record.address, "osmo1z98eg2ztdp2glyla62629nrlvczg8s7f8sgpm5");
    }

    #[tokio::test]
    async fn test_resolve_lens() {
        let current_dir = env::current_dir().unwrap();
        let path = current_dir.join("../../Settings.yaml");
        let settings = Settings::new_setting_path(path).unwrap();
        let client = LensClient::new(settings.name.lens.url);
        let address = client.resolve(&NameQuery::new("stani.lens"), Chain::Ethereum).await.unwrap();
        assert_eq!(address, "0xAd2c0BEAdE60fb9f7ec5C87bDE8e4c126145F6E7");
    }

    #[tokio::test]
    async fn test_resolve_suins() {
        let nodes = get_nodes_for_chain(Chain::Sui);
        let client = SuinsClient::new(nodes[0].url.clone());
        let address_result = client.resolve(&NameQuery::new("alpha.sui"), Chain::Sui).await;
        assert_eq!(address_result.unwrap(), "0x54e5c2a6f1276ac2ff623ac54e53e5a61a576906b3ec42fac8fe8bf5615d0957");
    }

    #[tokio::test]
    async fn test_resolve_aptos_name() {
        let current_dir = env::current_dir().unwrap();
        let path = current_dir.join("../../Settings.yaml");
        let settings = Settings::new_setting_path(path).unwrap();
        let client = AptosClient::new(settings.name.aptos.url);
        let address = client.resolve(&NameQuery::new("petra.apt"), Chain::Aptos).await.unwrap();
        assert_eq!(address, "0xfe2ffdb3a74307f7314a1c8ab3762b6b5869a3c1278cdd5d230249453e15a1db");
    }

    #[tokio::test]
    async fn test_resolve_hlnames() {
        let current_dir = env::current_dir().unwrap();
        let path = current_dir.join("../../Settings.yaml");
        let settings = Settings::new_setting_path(path).unwrap();
        let client = Hyperliquid::new(settings.name.hyperliquid.url);
        let name = "TESTOOOR.HL";
        let address = client.resolve(&NameQuery::new(name), Chain::Ethereum).await.unwrap();
        assert_eq!(address, "0xb43f5153B1c867BF78ACB3C35aa9b8ae366415c5");

        let address = client.resolve(&NameQuery::new(name), Chain::Hyperliquid).await.unwrap();
        assert_eq!(address, "0xF26F5551E96aE5162509B25925fFfa7F07B2D652");

        let address = client.resolve(&NameQuery::new(name), Chain::Solana).await.unwrap();
        assert_eq!(address, "CKAvaYmwqCbg8nZCUCNj6Cvr11HauALtNoGT7WirPoAp");
    }

    #[tokio::test]
    async fn test_resolve_alldomains() {
        let nodes = get_nodes_for_chain(Chain::Solana);
        let client = AllDomainsClient::new(nodes[0].url.clone());
        let address = client.resolve(&NameQuery::new("miester.poor"), Chain::Solana).await.unwrap();
        assert_eq!(address.trim(), "2EGGxj2qbNAJNgLCPKca8sxZYetyTjnoRspTPjzN2D67");
    }

    #[tokio::test]
    async fn test_resolve_near_account() {
        let nodes = get_nodes_for_chain(Chain::Near);
        let client = NearNameClient::new(nodes[0].url.clone());
        let address = client.resolve(&NameQuery::new("wrap.near"), Chain::Near).await.unwrap();
        assert_eq!(address, "wrap.near");
    }
}
