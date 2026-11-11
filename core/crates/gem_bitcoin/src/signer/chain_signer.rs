use primitives::{BitcoinChain, ChainSigner, SignerError, SignerInput, TransferDataOutputAction};

use crate::signer::{
    planner::UtxoPlanner,
    psbt::{BitcoinPsbtRequest, finalize_psbt},
    transaction::sign_plan,
};

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

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        if !matches!(self.chain, BitcoinChain::Bitcoin) {
            return SignerError::invalid_input_err(format!("{} does not sign PSBTs", self.chain.get_chain()));
        }
        let extra = input.input_type.get_generic_data().map_err(SignerError::invalid_input)?;
        let request = BitcoinPsbtRequest::parse(extra.data_as_str().map_err(SignerError::invalid_input)?)?;
        let signed = request.sign(&input.sender_address, private_key, &input.metadata.get_utxos()?)?;
        match extra.output_action {
            TransferDataOutputAction::Sign => Ok(signed.to_string()),
            TransferDataOutputAction::Send => finalize_psbt(signed),
        }
    }
}
