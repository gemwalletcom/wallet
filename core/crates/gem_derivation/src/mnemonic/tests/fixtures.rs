use primitives::Chain;

pub(super) use primitives::testkit::ABANDON_PHRASE as PHRASE;
const LOCAL_KEYSTORE_TEST_PHRASE: &str = "shoot island position soft burden budget tooth cruel issue economy destroy above";
pub(super) const TEST_PHRASE: &str = "seminar cruel gown pause law tortoise step stairs size amused pond weapon";

pub(super) struct DerivationExpectation {
    pub address: &'static str,
    pub path: &'static str,
}

pub(super) struct BitcoinFamilyV3Vector {
    pub phrase: &'static str,
    pub chain: Chain,
    pub address: &'static str,
    pub extended_public_key: &'static str,
}

pub(super) const BITCOIN_FAMILY_V3_VECTORS: &[BitcoinFamilyV3Vector] = &[
    BitcoinFamilyV3Vector {
        phrase: LOCAL_KEYSTORE_TEST_PHRASE,
        chain: Chain::Bitcoin,
        address: "bc1quvuarfksewfeuevuc6tn0kfyptgjvwsvrprk9d",
        extended_public_key: "zpub6r7CMEBpuNt4XH7pAVT2eH5kcz3S6w37AXHRpBfNPc4tAVuyaX4dahz5zs5knwVQH26bLKUDj3webt2NjpYD3i6mqqZ3Xa7vnaS4WW13fGW",
    },
    BitcoinFamilyV3Vector {
        phrase: LOCAL_KEYSTORE_TEST_PHRASE,
        chain: Chain::BitcoinCash,
        address: "qpzl3jxkzgvfd9flnd26leud5duv795fnv7vuaha70",
        extended_public_key: "xpub6CdXPxcVxcAJDED1NgMQnoDk5btzaLZy6W2BKvRXpXvVWib8w7nzmTfH1bLpbk4wTxTg9Nb1NSa3F28UYJqG7f51RezF9dRDP8K6iEkBTqB",
    },
    BitcoinFamilyV3Vector {
        phrase: LOCAL_KEYSTORE_TEST_PHRASE,
        chain: Chain::Litecoin,
        address: "ltc1qhd8fxxp2dx3vsmpac43z6ev0kllm4n53t5sk0u",
        extended_public_key: "zpub6rZRpnf2TEQS8C4tEEYZ9SqXZVxrDaPBj5prm3JCmr7AxdGKyxx3SghwQrd8KhpS9qy7ZRZeu9ugya1o3hAv9NneSpMwrzGcLTjVR4WHZ1P",
    },
    BitcoinFamilyV3Vector {
        phrase: LOCAL_KEYSTORE_TEST_PHRASE,
        chain: Chain::Doge,
        address: "DJRFZNg8jkUtjcpo2zJd92FUAzwRjitw6f",
        extended_public_key: "dgub8sRb9TVDFn6uAqq5cqGQWftQ4JCHxdDCBd34f3WtAqprKW89Fiajv4g9fRMEFoxNE3pjRUYTje3thTmNurC2hRYVcu8PUsqRUq8QkPYXCqs",
    },
    BitcoinFamilyV3Vector {
        phrase: LOCAL_KEYSTORE_TEST_PHRASE,
        chain: Chain::Zcash,
        address: "t1YYnByMzdGhQv3W3rnjHMrJs6HH4Y231gy",
        extended_public_key: "xpub6C9VfHLXZrdRSkBE7MnfvGEkk9eZgSB6rTi2DVpiykKgZYTvZtKcHZKCfZRmJKXvZzqrtWSNVwJGSs9uTvy3bVbSkb4o9g8NN6GMBvGE9cm",
    },
];

pub(super) fn expected_derivation(chain: Chain) -> DerivationExpectation {
    match chain {
        Chain::Ethereum
        | Chain::SmartChain
        | Chain::Polygon
        | Chain::Arbitrum
        | Chain::Optimism
        | Chain::Base
        | Chain::AvalancheC
        | Chain::OpBNB
        | Chain::Fantom
        | Chain::Gnosis
        | Chain::Manta
        | Chain::Blast
        | Chain::ZkSync
        | Chain::Linea
        | Chain::Mantle
        | Chain::Celo
        | Chain::World
        | Chain::Sonic
        | Chain::SeiEvm
        | Chain::Abstract
        | Chain::Berachain
        | Chain::Ink
        | Chain::Unichain
        | Chain::Hyperliquid
        | Chain::HyperCore
        | Chain::Monad
        | Chain::Plasma
        | Chain::XLayer
        | Chain::Stable => DerivationExpectation {
            address: "0x9858EfFD232B4033E47d90003D41EC34EcaEda94",
            path: "m/44'/60'/0'/0/0",
        },
        Chain::Solana => DerivationExpectation {
            address: "HAgk14JpMQLgt6rVgv7cBQFJWFto5Dqxi472uT3DKpqk",
            path: "m/44'/501'/0'/0'",
        },
        Chain::Tron => DerivationExpectation {
            address: "TUEZSdKsoDHQMeZwihtdoBiN46zxhGWYdH",
            path: "m/44'/195'/0'/0/0",
        },
        Chain::Thorchain => DerivationExpectation {
            address: "thor1gm00vwsfcp48enm4uv9e5dhm37jtd0ye27wrx0",
            path: "m/44'/931'/0'/0/0",
        },
        Chain::Mayachain => DerivationExpectation {
            address: "maya1gm00vwsfcp48enm4uv9e5dhm37jtd0ye2fs0sl",
            path: "m/44'/931'/0'/0/0",
        },
        Chain::Cosmos => DerivationExpectation {
            address: "cosmos19rl4cm2hmr8afy4kldpxz3fka4jguq0auqdal4",
            path: "m/44'/118'/0'/0/0",
        },
        Chain::Osmosis => DerivationExpectation {
            address: "osmo19rl4cm2hmr8afy4kldpxz3fka4jguq0a5m7df8",
            path: "m/44'/118'/0'/0/0",
        },
        Chain::Ton => DerivationExpectation {
            address: "UQAzWZa6nM5mJev91wGc7VCSfBoIsYRqKJpV78N8Add9-RKY",
            path: "m/44'/607'/0'",
        },
        Chain::Aptos => DerivationExpectation {
            address: "0xeb663b681209e7087d681c5d3eed12aaa8e1915e7c87794542c3f96e94b3d3bf",
            path: "m/44'/637'/0'/0'/0'",
        },
        Chain::Sui => DerivationExpectation {
            address: "0x5e93a736d04fbb25737aa40bee40171ef79f65fae833749e3c089fe7cc2161f1",
            path: "m/44'/784'/0'/0'/0'",
        },
        Chain::Near => DerivationExpectation {
            address: "5510e2b44cae6eb807e3e0e45d579dda058c274abcba15e5cb84636f5d1ee412",
            path: "m/44'/397'/0'",
        },
        Chain::Stellar => DerivationExpectation {
            address: "GB3JDWCQJCWMJ3IILWIGDTQJJC5567PGVEVXSCVPEQOTDN64VJBDQBYX",
            path: "m/44'/148'/0'",
        },
        Chain::Algorand => DerivationExpectation {
            address: "EP2D7TV7IAFANZHK3B6QLKB53N5UTD7RARVXZTWCPCRQQBKYVGM2XIMT2Q",
            path: "m/44'/283'/0'/0'/0'",
        },
        Chain::Xrp => DerivationExpectation {
            address: "rHsMGQEkVNJmpGWs8XUBoTBiAAbwxZN5v3",
            path: "m/44'/144'/0'/0/0",
        },
        Chain::Celestia => DerivationExpectation {
            address: "celestia19rl4cm2hmr8afy4kldpxz3fka4jguq0ad2ud9c",
            path: "m/44'/118'/0'/0/0",
        },
        Chain::Injective => DerivationExpectation {
            address: "inj1npvwllfr9dqr8erajqqr6s0vxnk2ak55re90dz",
            path: "m/44'/60'/0'/0/0",
        },
        Chain::Sei => DerivationExpectation {
            address: "sei19rl4cm2hmr8afy4kldpxz3fka4jguq0a3vute5",
            path: "m/44'/118'/0'/0/0",
        },
        Chain::Noble => DerivationExpectation {
            address: "noble19rl4cm2hmr8afy4kldpxz3fka4jguq0a5rc48m",
            path: "m/44'/118'/0'/0/0",
        },
        Chain::Polkadot => DerivationExpectation {
            address: "14E9StbjYhJiAfsNMEcq5tETq79Q6EqaGyebdziY214hNWDH",
            path: "m/44'/354'/0'/0'/0'",
        },
        Chain::Cardano => DerivationExpectation {
            address: "addr1qy8ac7qqy0vtulyl7wntmsxc6wex80gvcyjy33qffrhm7sh927ysx5sftuw0dlft05dz3c7revpf7jx0xnlcjz3g69mq4afdhv",
            path: "m/1852'/1815'/0'/0/0",
        },
        Chain::Bitcoin => DerivationExpectation {
            address: "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu",
            path: "m/84'/0'/0'/0/0",
        },
        Chain::BitcoinCash => DerivationExpectation {
            address: "qqyx49mu0kkn9ftfj6hje6g2wfer34yfnq5tahq3q6",
            path: "m/44'/145'/0'/0/0",
        },
        Chain::Litecoin => DerivationExpectation {
            address: "ltc1qjmxnz78nmc8nq77wuxh25n2es7rzm5c2rkk4wh",
            path: "m/84'/2'/0'/0/0",
        },
        Chain::Doge => DerivationExpectation {
            address: "DBus3bamQjgJULBJtYXpEzDWQRwF5iwxgC",
            path: "m/44'/3'/0'/0/0",
        },
        Chain::Zcash => DerivationExpectation {
            address: "t1XVXWCvpMgBvUaed4XDqWtgQgJSu1Ghz7F",
            path: "m/44'/133'/0'/0/0",
        },
    }
}
