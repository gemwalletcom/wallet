use gem_algorand::AlgorandAddress;
use gem_aptos::AccountAddress as AptosAddress;
use gem_bitcoin::BitcoinAddress;
use gem_evm::EthereumAddress;
use gem_polkadot::PolkadotAddress;
use gem_solana::SolanaAddress;
use gem_stellar::StellarAddress;
use gem_sui::address::SuiAddress;
use gem_ton::Address as TonAddress;
use gem_tron::address::TronAddress;
use gem_xrp::XrpAddress;
use primitives::{Account, Address, BitcoinChain, Chain, ChainType};

pub(crate) fn account_matches_address(account: &Account, address: &str) -> bool {
    let expected = account.address.as_str();
    match account.chain.chain_type() {
        ChainType::Ethereum | ChainType::HyperCore => parsed_addresses_match(expected, address, EthereumAddress::try_parse),
        ChainType::Solana => parsed_addresses_match(expected, address, SolanaAddress::try_parse),
        ChainType::Ton => parsed_addresses_match(expected, address, TonAddress::try_parse),
        ChainType::Tron => parsed_addresses_match(expected, address, TronAddress::try_parse),
        ChainType::Aptos => parsed_addresses_match(expected, address, AptosAddress::try_parse),
        ChainType::Sui => parsed_addresses_match(expected, address, SuiAddress::try_parse),
        ChainType::Stellar => parsed_addresses_match(expected, address, StellarAddress::try_parse),
        ChainType::Algorand => parsed_addresses_match(expected, address, AlgorandAddress::try_parse),
        ChainType::Xrp => parsed_addresses_match(expected, address, XrpAddress::try_parse),
        ChainType::Polkadot => parsed_addresses_match(expected, address, PolkadotAddress::try_parse),
        ChainType::Bitcoin => BitcoinChain::from_chain(account.chain).is_some_and(|chain| parsed_addresses_match(expected, address, |address| BitcoinAddress::try_parse_for_chain(address, chain))),
        ChainType::Cosmos | ChainType::Near | ChainType::Cardano => expected == address,
    }
}

fn parsed_addresses_match<T: Address>(left: &str, right: &str, parse: impl Fn(&str) -> Option<T>) -> bool {
    parse(left).zip(parse(right)).is_some_and(|(left, right)| left.as_bytes() == right.as_bytes())
}

pub fn validate_address(address: &str, chain: Chain) -> bool {
    match chain.chain_type() {
        ChainType::Ethereum | ChainType::HyperCore => gem_evm::validate_address(address),
        ChainType::Solana => gem_solana::validate_address(address),
        ChainType::Cosmos => gem_cosmos::validate_address(address, chain),
        ChainType::Ton => gem_ton::validate_address(address),
        ChainType::Tron => gem_tron::validate_address(address),
        ChainType::Aptos => gem_aptos::validate_address(address),
        ChainType::Sui => gem_sui::validate_address(address),
        ChainType::Near => gem_near::is_valid_address(address),
        ChainType::Stellar => gem_stellar::validate_address(address),
        ChainType::Algorand => gem_algorand::validate_address(address),
        ChainType::Xrp => gem_xrp::validate_address(address),
        ChainType::Polkadot => gem_polkadot::validate_address(address),
        ChainType::Bitcoin => gem_bitcoin::validate_address(address, chain),
        ChainType::Cardano => gem_cardano::validate_address(address),
    }
}

#[cfg(test)]
mod tests {
    use primitives::testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_EVM_SENDER, TEST_SOLANA_SENDER, TEST_TON_SENDER};

    use super::*;

    #[test]
    fn test_account_matches_address_evm() {
        for chain in [Chain::Ethereum, Chain::HyperCore] {
            let account = Account::mock(chain, TEST_EVM_SENDER);
            assert!(account_matches_address(&account, &TEST_EVM_SENDER.to_lowercase()));
            assert!(account_matches_address(&account, TEST_EVM_SENDER.trim_start_matches("0x")));
            assert!(!account_matches_address(&account, &format!(" {TEST_EVM_SENDER}\n")));
            assert!(!account_matches_address(&account, TEST_EVM_RECIPIENT));
            assert!(!account_matches_address(&account, "invalid"));
            assert!(!account_matches_address(&Account::mock(chain, "invalid"), TEST_EVM_SENDER));
            assert!(!account_matches_address(&Account::mock(chain, ""), ""));
        }
    }

    #[test]
    fn test_account_matches_address_ton() {
        let account = Account::mock(Chain::Ton, TEST_TON_SENDER);
        let address = TonAddress::try_parse(TEST_TON_SENDER).unwrap();
        assert!(account_matches_address(&account, &address.encode_bounceable()));
        assert!(account_matches_address(&account, &format!("{}:{}", address.workchain(), hex::encode(address.hash_part()))));
        assert!(!account_matches_address(&account, &TonAddress::new(-1, *address.hash_part()).encode()));
        assert!(!account_matches_address(&account, "invalid"));
    }

    #[test]
    fn test_account_matches_address_bitcoin() {
        let bitcoin = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
        let bitcoin_cash = "qpzl3jxkzgvfd9flnd26leud5duv795fnv7vuaha70";
        let account = Account::mock(Chain::Bitcoin, bitcoin);
        assert!(account_matches_address(&account, &bitcoin.to_uppercase()));
        assert!(account_matches_address(&Account::mock(Chain::BitcoinCash, bitcoin_cash), &format!("bitcoincash:{bitcoin_cash}")));
        assert!(!account_matches_address(&Account::mock(Chain::Litecoin, bitcoin), bitcoin));
        assert!(!account_matches_address(&account, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"));
    }

    #[test]
    fn test_account_matches_address_case_sensitive() {
        let account = Account::mock(Chain::Solana, TEST_SOLANA_SENDER);
        assert!(account_matches_address(&account, TEST_SOLANA_SENDER));
        assert!(!account_matches_address(&account, &TEST_SOLANA_SENDER.to_lowercase()));
    }

    #[test]
    fn test_account_matches_address_parsed_hex() {
        let address = "ada112cfb90b44ba889cc5d39ac2bf46281e4a91f7919c693bcd9b8323e81ed2";
        for chain in [Chain::Aptos, Chain::Sui] {
            let account = Account::mock(chain, &format!("0x{address}"));
            assert!(account_matches_address(&account, &format!("0x{}", address.to_uppercase())));
            assert!(!account_matches_address(&account, &format!("0x{}", "ab".repeat(32))));
        }
    }

    #[test]
    fn test_account_matches_address_rejects_unparseable_addresses() {
        for chain in [
            Chain::Ethereum,
            Chain::HyperCore,
            Chain::Bitcoin,
            Chain::Ton,
            Chain::Solana,
            Chain::Tron,
            Chain::Aptos,
            Chain::Sui,
            Chain::Stellar,
            Chain::Algorand,
            Chain::Xrp,
            Chain::Polkadot,
        ] {
            assert!(!account_matches_address(&Account::mock(chain, "invalid"), "invalid"), "{chain}");
        }
    }

    #[test]
    fn test_chain_address_validation() {
        assert!(validate_address("0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a", Chain::Ethereum));
        assert!(!validate_address("0X5615e8ab93b9d695b6d4d6545f7792aa59e1069a", Chain::Ethereum));
        assert!(validate_address("cosmos1h3laqcrmul79zwtw6j63ncsl0adfj07wgupylj", Chain::Cosmos));
        assert!(validate_address("GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2", Chain::Solana));
        assert!(validate_address("rnBFvgZphmN39GWzUJeUitaP22Fr9be75H", Chain::Xrp));
        assert!(!validate_address("rnBFvgZphmN39GWzUJeUitaP22Fr9be75J", Chain::Xrp));
        assert!(validate_address("h3rman.near", Chain::Near));
        assert!(validate_address("0x85f17cf997934a597031b2e18a9ab6ebd4b9f6a4", Chain::Near));
        assert!(validate_address("UQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz3VV", Chain::Ton));
        assert!(validate_address("15e6w4u9nH4Tb9HdJco2Zua4y5DpHb1hHXBKBGkUrLMTpuXo", Chain::Polkadot));
        assert!(!validate_address("15e6w4u9nH4Tb9HdJco2Zua4y5DpHb1hHXBKBGkUrLMTpuXj", Chain::Polkadot));
        assert!(validate_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4", Chain::Bitcoin));
        assert!(!validate_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4", Chain::Litecoin));
        assert!(validate_address("qpzl3jxkzgvfd9flnd26leud5duv795fnv7vuaha70", Chain::BitcoinCash));
        assert!(validate_address("bitcoincash:qpzl3jxkzgvfd9flnd26leud5duv795fnv7vuaha70", Chain::BitcoinCash));
        assert!(validate_address("addr1q8043m5heeaydnvtmmkyuhe6qv5havvhsf0d26q3jygsspxlyfpyk6yqkw0yhtyvtr0flekj84u64az82cufmqn65zdsylzk23", Chain::Cardano));
        assert!(!validate_address("addr_test1qr4p6f6mm0q9kfyyd9u30umk9cc6gk0nxu25k5rsc4fp7ls7k0qqxslcwwj4gvn4yfmdyrfgwjt3ztuz4zpy4242u0m95r0n", Chain::Cardano));
    }
}
