use gem_tron::address::TronAddress;
use primitives::{Chain, ChainType};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Contracts {
    pub targets: Vec<&'static str>,
    pub spenders: Vec<&'static str>,
}

impl Contracts {
    pub fn new(targets: Vec<&'static str>) -> Self {
        Self { spenders: targets.clone(), targets }
    }

    pub fn universal_router(router: &'static str, permit2: &'static str) -> Self {
        Self {
            targets: vec![router],
            spenders: vec![permit2, router],
        }
    }
}

impl FromIterator<Contracts> for Contracts {
    fn from_iter<I: IntoIterator<Item = Contracts>>(contracts: I) -> Self {
        contracts.into_iter().fold(Contracts::default(), |mut merged, contracts| {
            merged.targets.extend(contracts.targets);
            merged.spenders.extend(contracts.spenders);
            merged
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Binding {
    Contracts(Contracts),
    DepositAddress,
    BuiltOnDevice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deployment {
    pub chain: Chain,
    pub contract: &'static str,
    pub spender: &'static str,
}

impl Deployment {
    pub const fn new(chain: Chain, contract: &'static str) -> Self {
        Self { chain, contract, spender: contract }
    }

    pub const fn with_spender(chain: Chain, contract: &'static str, spender: &'static str) -> Self {
        Self { chain, contract, spender }
    }

    pub fn contracts(deployments: &[Self], chain: Chain) -> Contracts {
        deployments
            .iter()
            .filter(|deployment| deployment.chain == chain)
            .map(|deployment| Contracts {
                targets: vec![deployment.contract],
                spenders: vec![deployment.spender],
            })
            .collect()
    }

    pub fn contains(chain: Chain, contracts: &[&str], address: &str) -> bool {
        contracts.iter().any(|contract| match chain.chain_type() {
            ChainType::Ethereum => contract.eq_ignore_ascii_case(address),
            ChainType::Tron => TronAddress::from_hex_or_base58(contract).is_some_and(|contract| TronAddress::from_hex_or_base58(address) == Some(contract)),
            ChainType::Bitcoin
            | ChainType::Solana
            | ChainType::Cosmos
            | ChainType::Ton
            | ChainType::Aptos
            | ChainType::Sui
            | ChainType::Xrp
            | ChainType::Near
            | ChainType::Stellar
            | ChainType::Algorand
            | ChainType::Polkadot
            | ChainType::Cardano
            | ChainType::HyperCore => *contract == address,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::contract_constants::TRON_SUNSWAP_PERMIT2_CONTRACT;

    #[test]
    fn test_contracts() {
        let deployments = [Deployment::with_spender(Chain::Ethereum, "0xrouter", "0xspender"), Deployment::new(Chain::Base, "0xvault")];

        assert_eq!(
            Deployment::contracts(&deployments, Chain::Ethereum),
            Contracts {
                targets: vec!["0xrouter"],
                spenders: vec!["0xspender"],
            }
        );
        assert_eq!(Deployment::contracts(&deployments, Chain::Base), Contracts::new(vec!["0xvault"]));
        assert_eq!(Deployment::contracts(&deployments, Chain::Solana), Contracts::default());
    }

    #[test]
    fn test_contains() {
        assert!(Deployment::contains(Chain::Ethereum, &["0xRouter"], "0xROUTER"));
        assert!(!Deployment::contains(Chain::Solana, &["program"], "PROGRAM"));
        assert!(!Deployment::contains(Chain::Ethereum, &[], "0xRouter"));
        let permit2_hex = alloy_primitives::Address::from(TronAddress::parse(TRON_SUNSWAP_PERMIT2_CONTRACT).unwrap()).to_checksum(None);
        assert!(Deployment::contains(Chain::Tron, &[TRON_SUNSWAP_PERMIT2_CONTRACT], &permit2_hex), "a Tron contract matches in base58 or hex");
        assert!(!Deployment::contains(Chain::Tron, &[TRON_SUNSWAP_PERMIT2_CONTRACT], "TXtEs6t2oUWQsNos7m68gbHdE9Q5n6x2oN"));
        assert!(!Deployment::contains(Chain::Tron, &[TRON_SUNSWAP_PERMIT2_CONTRACT], "not an address"));
    }
}
