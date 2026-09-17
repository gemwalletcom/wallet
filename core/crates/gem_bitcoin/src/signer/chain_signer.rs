use primitives::{BitcoinChain, ChainSigner, SignerError, SignerInput};

use crate::signer::{
    planner::{SpendPlan, SpendRequest, UtxoPlanner},
    transaction::sign_plan,
};

pub struct BitcoinChainSigner {
    chain: BitcoinChain,
}

impl BitcoinChainSigner {
    pub fn new(chain: BitcoinChain) -> Self {
        Self { chain }
    }

    fn sign_request(&self, request: SpendRequest, private_key: &[u8], zcash_branch_id: Option<u32>) -> Result<String, SignerError> {
        let plan: SpendPlan = UtxoPlanner::plan(request)?;
        sign_plan(self.chain, &plan, private_key, zcash_branch_id)
    }
}

impl ChainSigner for BitcoinChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        self.sign_request(SpendRequest::transfer(self.chain, input)?, private_key, input.metadata.get_zcash_branch_id())
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        Ok(vec![self.sign_request(
            SpendRequest::swap(self.chain, input)?,
            private_key,
            input.metadata.get_zcash_branch_id(),
        )?])
    }
}
