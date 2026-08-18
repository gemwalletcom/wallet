use primitives::{Chain, NodeCheckProfile, NodeCheckRequest};

const DEFAULT_EVM_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";

pub fn node_check_request(chain: Chain, profile: NodeCheckProfile) -> NodeCheckRequest {
    match profile {
        NodeCheckProfile::Basic => NodeCheckRequest::Basic,
        NodeCheckProfile::Parser => NodeCheckRequest::Parser,
        NodeCheckProfile::Wallet => wallet_node_check_request(chain),
    }
}

fn wallet_node_check_request(chain: Chain) -> NodeCheckRequest {
    let (address, transaction_id) = match chain {
        Chain::Bitcoin => (
            "bc1qk9cu0nj5czvalnvmlsyc8tmqh8d6f0v9plrrdr",
            Some("654c6a28f7ff1915d2b9abc2e18e32a37e0196203d64aced6221651f003f5e94"),
        ),
        Chain::BitcoinCash => (
            "qpcns7lget89x9km0t8ry5fk52e8lhl53q0a64gd65",
            Some("8d8da67e2a629e30520f105d9f8bb49568e691b1da862d9c01e6c4037a934210"),
        ),
        Chain::Litecoin => (
            "ltc1qanrz523v9cxw5ng6unlkwfn3yzqllmxzyrd4zs",
            Some("a4098d7e973f72b611577717cd1a9e270bbcee1bcbf2f991205236811caa0b01"),
        ),
        Chain::Ethereum => (DEFAULT_EVM_ADDRESS, Some("0x98dd4d9a586620f84e8066f1b015d663f9c0c94c4e0e02377840c3e6d43e2ad3")),
        Chain::SmartChain => (DEFAULT_EVM_ADDRESS, Some("0xa9f6e1d1a02ba5bb5aa9b3c83773ef9ac6d8fe9abb1fa4512d422f0194d5d833")),
        Chain::Solana => (
            "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H",
            Some("4dHnggcXjvmMJY2J6iGqse12PeCYQzuTySgwJa36K8MuntmwNrCNztvYRX5ZGpQXzKjaf7g5vaZM7LTuXLNbi2Zx"),
        ),
        Chain::Polygon => (DEFAULT_EVM_ADDRESS, Some("0x3d4eb72380e6095d0667c6ec3420719dbec7d1d8b1628464a03ee6850ee716ed")),
        Chain::Thorchain => ("thor1gjkawwc7m9ena873fgckvm20f5wh556a3ug890", None),
        Chain::Mayachain => ("maya1cvh8mpz04az0x7vht6h6ekksg8wd650rh4cuda", None),
        Chain::Cosmos => ("cosmos1cvh8mpz04az0x7vht6h6ekksg8wd650r39ltwj", None),
        Chain::Osmosis => ("osmo1cvh8mpz04az0x7vht6h6ekksg8wd650re7vmcq", None),
        Chain::Arbitrum => (DEFAULT_EVM_ADDRESS, Some("0x6a38409d346190d38a28be23db35dcda5dc88df0de99c23049c967c388359857")),
        Chain::Ton => ("UQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz3VV", Some("gyjq/7IJ5KpSvZlnwixaS3RjI2xk1+5pup0k++S/yXY=")),
        Chain::Tron => ("TFdTEn9dJuqh351y8fyJ3eMmghFsZNwakb", None),
        Chain::Doge => (
            "DMKhUaRmnxJXfDxyFguMnMjVdgvnNipFzt",
            Some("a3d087183ce022bb679682aa989589bb0456180f34ed5daa96f8e4988b26968b"),
        ),
        Chain::Zcash => (
            "t1bGQcsCLuyYJyRHep6pKwtMYdi1XFrvjho",
            Some("b1038ceffc1ce6eeab15d9254e7fe84e81808e1d40affba2b8792b21314c1f03"),
        ),
        Chain::Optimism => (DEFAULT_EVM_ADDRESS, Some("0xc4edd56597745ae8fc8486b2cdf003ea52d9b37b0f72361eff3b5d73d62ae731")),
        Chain::Aptos => ("0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc", None),
        Chain::Base => (DEFAULT_EVM_ADDRESS, Some("0xb7f529ed53a7f716976cd53520677260b53edf011da7573374ccf8705b6b4a8e")),
        Chain::AvalancheC => (DEFAULT_EVM_ADDRESS, Some("0x64317b42490640403cb5a1c0c9c8672a7aa6f0216f372be8113d1ea84ad7ce0d")),
        Chain::Sui => ("0x93f65b8c16c263343bbf66cf9f8eef69cb1dbc92d13f0c331b0dcaeb76b4aab6", None),
        Chain::Xrp => (
            "rnZmVGX6f4pUYyS4oXYJzoLdRojQV8y297",
            Some("474F58E6C78F1DE8542036AB3C16E2B5A4089241DEE3E58142154DC3CA0E8271"),
        ),
        Chain::OpBNB => (DEFAULT_EVM_ADDRESS, Some("0x8581e4d41399e899fcf0e828b3b986b45854375d617ce5abc565afbd54741955")),
        Chain::Fantom => (DEFAULT_EVM_ADDRESS, Some("0x2c2c6b8a00eab2a8d948ee5ecf95730642ce03230870fe4e24657bfdff170254")),
        Chain::Gnosis => (DEFAULT_EVM_ADDRESS, Some("0x3b6f77ef3007b5e54fe8de3b3bcda971528b35eda0669e4893a97b6a35a4c31c")),
        Chain::Celestia => ("celestia1cvh8mpz04az0x7vht6h6ekksg8wd650rq0wm5l", None),
        Chain::Injective => ("inj1cvh8mpz04az0x7vht6h6ekksg8wd650rmvg0u2", None),
        Chain::Sei => ("sei1cvh8mpz04az0x7vht6h6ekksg8wd650rufwagn", None),
        Chain::SeiEvm => (DEFAULT_EVM_ADDRESS, Some("0x4fc879341cb99aeb24ef2388176bc0915a412273ff3fe93b905902adb64d949d")),
        Chain::Manta => (DEFAULT_EVM_ADDRESS, Some("0xc8aabd35fc1e43dde16709b2d489569202c47c273e3f59c7cbb5df8f9b0fe65a")),
        Chain::Blast => (DEFAULT_EVM_ADDRESS, Some("0xf81fffef507b5a18f073f701f4cf0df050cdfab2e0d4869be8a186bb61e626a4")),
        Chain::Noble => ("noble1cvh8mpz04az0x7vht6h6ekksg8wd650rex2rku", None),
        Chain::ZkSync => (DEFAULT_EVM_ADDRESS, Some("0x863aa2a481a309574009c53f2449bb21f9adb9d59bc56b4835d8f785c529fc02")),
        Chain::Linea => (DEFAULT_EVM_ADDRESS, Some("0x4cd8dba40e71cdf21fc6da8020a6e75d98e549ec31c5bb5ce6e8929638cf9c7f")),
        Chain::Mantle => (DEFAULT_EVM_ADDRESS, Some("0xf968326c238982141a97bca543f184f28e71d8db95882662558b4edc5476b30d")),
        Chain::Celo => (DEFAULT_EVM_ADDRESS, Some("0xa6dcede6af9e3c0324971790bb03e07c820c13f84396e71864ed3dd5643e8e12")),
        Chain::Near => ("051d30e6c78c4cf858389d62af5f703275450d318b85ff52a4ac963948cfdf95", None),
        Chain::World => (DEFAULT_EVM_ADDRESS, Some("0x6bc975455d9552086286e75b5be6351d2b29f9b8be061f289cadc1ce5ca1de8f")),
        Chain::Stellar => ("GAN2JTIWVKGZIDN5R2AFYLUV4IUXLBG3MQA3R5ECIIM5RUYT74Y3LDOP", None),
        Chain::Sonic => (DEFAULT_EVM_ADDRESS, Some("0x46cffcb41f25a43ea91f05704eeb27bc45391f616e1bf7e2e30ace5ce263ceac")),
        Chain::Algorand => ("RXIOUIR5IGFZMIZ7CR7FJXDYY4JI7NZG5UCWCZZNWXUPFJRLG6K6X5ITXM", None),
        Chain::Polkadot => ("125YLEK39toTQkLLHA6V4zqc7ixh9VuE4XFbcJfqAByB1pkM", None),
        Chain::Plasma => (DEFAULT_EVM_ADDRESS, Some("0x6d83a79e228ddaa04107afb03cfd1b1b74b24429d322d8e79d756e559895d3a8")),
        Chain::Cardano => (
            "addr1q84jz28nx62e2xp084xvgaqfmptca9ljem5yjvvm29n4d9kxp99pxjz0zfy52fep9mkyhcq995q8lpydka3lle58jghsy3zmph",
            None,
        ),
        Chain::Abstract => (DEFAULT_EVM_ADDRESS, Some("0xe064ad2d215da437b8496a95fc6d6b1124930599ca1eabb9bad515921e666105")),
        Chain::Berachain => (DEFAULT_EVM_ADDRESS, Some("0x6ce80fa54e067a9b36c7280eb93323b588636942805ef3643dd659c070b655bd")),
        Chain::Ink => (DEFAULT_EVM_ADDRESS, Some("0x1e455c14cf075a83e2fb5bbd165ff53cc0eb1699709bdb665f709f8560503527")),
        Chain::Unichain => (DEFAULT_EVM_ADDRESS, Some("0x2f931c88701faffc04dd65d5d05857dbaa76ec43a62116c6a69071c827d9c99e")),
        Chain::Hyperliquid => (DEFAULT_EVM_ADDRESS, Some("0x4785e5c28dbc8ec640b00a4985cf518926a5364a6843a48fe0e84edee3952093")),
        Chain::HyperCore => (DEFAULT_EVM_ADDRESS, None),
        Chain::Monad => (DEFAULT_EVM_ADDRESS, Some("0xae2fe7ab7d6920d84b78126dc2ce82a1e227e4f70bd7f037c3747396d5a73c57")),
        Chain::XLayer => (DEFAULT_EVM_ADDRESS, Some("0xa6e649c54eaf86b5bb51e0230bf97499ff348e2e5e6527aaddc55183b7ec8211")),
        Chain::Robinhood => (DEFAULT_EVM_ADDRESS, Some("0xdd81e20bb08437587dc6f6e2a7f0d43bd96101ca51f051c42806a307636f10db")),
        Chain::Stable => (DEFAULT_EVM_ADDRESS, Some("0x312b2a62ab4927fc7805789184f7e87c8e2e1e87c6eaa01706e58a979a54d4df")),
        Chain::Tempo => (DEFAULT_EVM_ADDRESS, Some("0x99649df228014eca4fe3058455b9bb30fbf700461daebb65e63251c180cccd85")),
    };

    NodeCheckRequest::Wallet {
        address: address.to_string(),
        transaction_id: transaction_id.map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use primitives::ChainType;

    use super::*;

    #[test]
    fn test_wallet_node_check_request() {
        for chain in Chain::all() {
            let NodeCheckRequest::Wallet { address, transaction_id } = node_check_request(chain, NodeCheckProfile::Wallet) else {
                panic!("wallet profile expected for {chain}");
            };
            assert!(!address.is_empty(), "wallet address missing for {chain}");
            if chain.chain_type() == ChainType::Ethereum {
                assert_eq!(address, DEFAULT_EVM_ADDRESS, "unexpected EVM wallet address for {chain}");
            }
            if matches!(chain.chain_type(), ChainType::Bitcoin | ChainType::Ethereum) {
                assert!(transaction_id.is_some(), "wallet transaction missing for {chain}");
            }
        }
    }
}
