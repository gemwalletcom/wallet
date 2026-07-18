use primitives::Chain;

pub(crate) struct NodeFixture {
    pub(crate) addresses: &'static [&'static str],
    pub(crate) transaction_ids: &'static [&'static str],
}

pub(crate) fn fixture(chain: Chain) -> Option<NodeFixture> {
    match chain {
        Chain::Ethereum => Some(NodeFixture {
            addresses: &["0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4"],
            transaction_ids: &["0x98dd4d9a586620f84e8066f1b015d663f9c0c94c4e0e02377840c3e6d43e2ad3"],
        }),
        Chain::SmartChain => Some(NodeFixture {
            addresses: &["0x2A49C84B7173e21f9116B2798735f87531526b36"],
            transaction_ids: &["0xa9f6e1d1a02ba5bb5aa9b3c83773ef9ac6d8fe9abb1fa4512d422f0194d5d833"],
        }),
        Chain::Polygon => Some(NodeFixture {
            addresses: &["0x2A49C84B7173e21f9116B2798735f87531526b36"],
            transaction_ids: &["0x3d4eb72380e6095d0667c6ec3420719dbec7d1d8b1628464a03ee6850ee716ed"],
        }),
        Chain::Plasma => Some(NodeFixture {
            addresses: &["0x8192bf75cb263e543c4f2c06edb983139034aa0f"],
            transaction_ids: &["0x6d83a79e228ddaa04107afb03cfd1b1b74b24429d322d8e79d756e559895d3a8"],
        }),
        Chain::Arbitrum => Some(NodeFixture {
            addresses: &["0x00000000000000000000000000000000000a4b05"],
            transaction_ids: &["0x6a38409d346190d38a28be23db35dcda5dc88df0de99c23049c967c388359857"],
        }),
        Chain::Optimism => Some(NodeFixture {
            addresses: &["0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001"],
            transaction_ids: &["0xc4edd56597745ae8fc8486b2cdf003ea52d9b37b0f72361eff3b5d73d62ae731"],
        }),
        Chain::Base => Some(NodeFixture {
            addresses: &["0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001"],
            transaction_ids: &["0xb7f529ed53a7f716976cd53520677260b53edf011da7573374ccf8705b6b4a8e"],
        }),
        Chain::AvalancheC => Some(NodeFixture {
            addresses: &["0xa36c8b1737195e634019fe27ae13d52d2e96947f"],
            transaction_ids: &["0x64317b42490640403cb5a1c0c9c8672a7aa6f0216f372be8113d1ea84ad7ce0d"],
        }),
        Chain::OpBNB => Some(NodeFixture {
            addresses: &["0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001"],
            transaction_ids: &["0x8581e4d41399e899fcf0e828b3b986b45854375d617ce5abc565afbd54741955"],
        }),
        Chain::Fantom => Some(NodeFixture {
            addresses: &["0x56730257ec944da158fdb3af7bbfbacabeaf9dbe"],
            transaction_ids: &["0x2c2c6b8a00eab2a8d948ee5ecf95730642ce03230870fe4e24657bfdff170254"],
        }),
        Chain::Gnosis => Some(NodeFixture {
            addresses: &["0x8c4c15870d27c1194b6893f6b94dd0ce9c2c8ba2"],
            transaction_ids: &["0x3b6f77ef3007b5e54fe8de3b3bcda971528b35eda0669e4893a97b6a35a4c31c"],
        }),
        Chain::Manta => Some(NodeFixture {
            addresses: &["0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001"],
            transaction_ids: &["0xc8aabd35fc1e43dde16709b2d489569202c47c273e3f59c7cbb5df8f9b0fe65a"],
        }),
        Chain::Blast => Some(NodeFixture {
            addresses: &["0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001"],
            transaction_ids: &["0xf81fffef507b5a18f073f701f4cf0df050cdfab2e0d4869be8a186bb61e626a4"],
        }),
        Chain::ZkSync => Some(NodeFixture {
            addresses: &["0x0baa722aefa911a4f7e7657198bcdb9efc06bf38"],
            transaction_ids: &["0x863aa2a481a309574009c53f2449bb21f9adb9d59bc56b4835d8f785c529fc02"],
        }),
        Chain::Linea => Some(NodeFixture {
            addresses: &["0x32c1e0876c6b2a907d06965d5625128daa4d893b"],
            transaction_ids: &["0x4cd8dba40e71cdf21fc6da8020a6e75d98e549ec31c5bb5ce6e8929638cf9c7f"],
        }),
        Chain::Mantle => Some(NodeFixture {
            addresses: &["0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001"],
            transaction_ids: &["0xf968326c238982141a97bca543f184f28e71d8db95882662558b4edc5476b30d"],
        }),
        Chain::Celo => Some(NodeFixture {
            addresses: &["0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001"],
            transaction_ids: &["0xa6dcede6af9e3c0324971790bb03e07c820c13f84396e71864ed3dd5643e8e12"],
        }),
        Chain::World => Some(NodeFixture {
            addresses: &["0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001"],
            transaction_ids: &["0x6bc975455d9552086286e75b5be6351d2b29f9b8be061f289cadc1ce5ca1de8f"],
        }),
        Chain::Sonic => Some(NodeFixture {
            addresses: &["0x7e62e6c99a80e28669a55fcef1316b78f97b4319"],
            transaction_ids: &["0x46cffcb41f25a43ea91f05704eeb27bc45391f616e1bf7e2e30ace5ce263ceac"],
        }),
        Chain::SeiEvm => Some(NodeFixture {
            addresses: &["0x028a9fd11fc977de04d7b509e0c7b1e22545c7f3"],
            transaction_ids: &["0x4fc879341cb99aeb24ef2388176bc0915a412273ff3fe93b905902adb64d949d"],
        }),
        Chain::Abstract => Some(NodeFixture {
            addresses: &["0x53244757268dada82a8064b6570651f0e30a647e"],
            transaction_ids: &["0xe064ad2d215da437b8496a95fc6d6b1124930599ca1eabb9bad515921e666105"],
        }),
        Chain::Berachain => Some(NodeFixture {
            addresses: &["0xfffffffffffffffffffffffffffffffffffffffe"],
            transaction_ids: &["0x6ce80fa54e067a9b36c7280eb93323b588636942805ef3643dd659c070b655bd"],
        }),
        Chain::Ink => Some(NodeFixture {
            addresses: &["0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001"],
            transaction_ids: &["0x1e455c14cf075a83e2fb5bbd165ff53cc0eb1699709bdb665f709f8560503527"],
        }),
        Chain::Unichain => Some(NodeFixture {
            addresses: &["0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001"],
            transaction_ids: &["0x2f931c88701faffc04dd65d5d05857dbaa76ec43a62116c6a69071c827d9c99e"],
        }),
        Chain::Hyperliquid => Some(NodeFixture {
            addresses: &["0xfe65cc490daf50ee9a0503669bd7ec465090c81c"],
            transaction_ids: &["0x4785e5c28dbc8ec640b00a4985cf518926a5364a6843a48fe0e84edee3952093"],
        }),
        Chain::Monad => Some(NodeFixture {
            addresses: &["0x6f49a8f621353f12378d0046e7d7e4b9b249dc9e"],
            transaction_ids: &["0xae2fe7ab7d6920d84b78126dc2ce82a1e227e4f70bd7f037c3747396d5a73c57"],
        }),
        Chain::XLayer => Some(NodeFixture {
            addresses: &["0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001"],
            transaction_ids: &["0xa6e649c54eaf86b5bb51e0230bf97499ff348e2e5e6527aaddc55183b7ec8211"],
        }),
        Chain::Robinhood => Some(NodeFixture {
            addresses: &["0x00000000000000000000000000000000000a4b05"],
            transaction_ids: &["0xdd81e20bb08437587dc6f6e2a7f0d43bd96101ca51f051c42806a307636f10db"],
        }),
        Chain::Stable => Some(NodeFixture {
            addresses: &["0x8888888888888888888888888888888888888888"],
            transaction_ids: &["0x312b2a62ab4927fc7805789184f7e87c8e2e1e87c6eaa01706e58a979a54d4df"],
        }),
        Chain::Solana => Some(NodeFixture {
            addresses: &["37BenMAXFJMo3GaXKb2XLsNQXmd6VbbdShZWnwDj9D6k"],
            transaction_ids: &["4dHnggcXjvmMJY2J6iGqse12PeCYQzuTySgwJa36K8MuntmwNrCNztvYRX5ZGpQXzKjaf7g5vaZM7LTuXLNbi2Zx"],
        }),
        Chain::Bitcoin
        | Chain::BitcoinCash
        | Chain::Litecoin
        | Chain::Thorchain
        | Chain::Mayachain
        | Chain::Cosmos
        | Chain::Osmosis
        | Chain::Ton
        | Chain::Tron
        | Chain::Doge
        | Chain::Zcash
        | Chain::Aptos
        | Chain::Sui
        | Chain::Xrp
        | Chain::Celestia
        | Chain::Injective
        | Chain::Sei
        | Chain::Noble
        | Chain::Near
        | Chain::Stellar
        | Chain::Algorand
        | Chain::Polkadot
        | Chain::Cardano
        | Chain::HyperCore => None,
    }
}
