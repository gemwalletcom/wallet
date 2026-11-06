use primitives::{BitcoinChain, ChainSigner, SignerError, SignerInput};

use crate::signer::{UtxoPlanner, transaction::sign_plan};

pub struct BitcoinChainSigner {
    chain: BitcoinChain,
}

impl BitcoinChainSigner {
    pub fn new(chain: BitcoinChain) -> Self {
        Self { chain }
    }
}

impl ChainSigner for BitcoinChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_plan(self.chain, &UtxoPlanner::transfer(self.chain, input)?, private_key)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        Ok(vec![sign_plan(self.chain, &UtxoPlanner::swap(self.chain, input)?, private_key)?])
    }
}
