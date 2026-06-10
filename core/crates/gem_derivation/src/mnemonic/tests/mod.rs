use std::collections::HashMap;

use primitives::Chain;
use serde::Deserialize;

mod derivation;

pub(super) use primitives::testkit::ABANDON_PHRASE as PHRASE;
pub(super) const TEST_PHRASE: &str = "seminar cruel gown pause law tortoise step stairs size amused pond weapon";

#[derive(Deserialize)]
pub(super) struct DerivationExpectation {
    pub address: String,
    pub path: String,
}

#[derive(Deserialize)]
pub(super) struct BitcoinFamilyV3Vector {
    pub phrase: String,
    pub chain: Chain,
    pub address: String,
    pub extended_public_key: String,
}

pub(super) fn bitcoin_family_v3_vectors() -> Vec<BitcoinFamilyV3Vector> {
    serde_json::from_str(include_str!("../../../testdata/bitcoin_family_v3_vectors.json")).unwrap()
}

pub(super) fn expected_derivation(chain: Chain) -> DerivationExpectation {
    let mut expectations: HashMap<Chain, DerivationExpectation> = serde_json::from_str(include_str!("../../../testdata/derivation_expectations.json")).unwrap();
    expectations.remove(&chain).unwrap_or_else(|| panic!("missing derivation expectation for chain {chain}"))
}
