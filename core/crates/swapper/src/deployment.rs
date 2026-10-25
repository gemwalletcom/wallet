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
            _ => *contract == address,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
