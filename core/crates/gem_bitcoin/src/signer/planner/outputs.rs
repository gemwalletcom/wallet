use bitcoin::{
    ScriptBuf,
    blockdata::{opcodes::all::OP_RETURN, script::Builder},
    script::PushBytesBuf,
};
use primitives::{BitcoinChain, SignerError};

use super::{PlanOutput, SpendDestination};

const MAX_OP_RETURN_BYTES: usize = 80;

#[derive(Debug, Clone)]
pub(super) enum Payment {
    Script { script_pubkey: ScriptBuf, memo_output: Option<PlanOutput> },
    External,
}

impl Payment {
    pub(super) fn new(destination: SpendDestination, memo: Option<&[u8]>) -> Result<Self, SignerError> {
        match destination {
            SpendDestination::Script(script_pubkey) => Ok(Self::Script {
                script_pubkey,
                memo_output: memo.map(op_return_output).transpose()?,
            }),
            SpendDestination::External => Ok(Self::External),
        }
    }

    pub(super) fn dust_threshold(&self, chain: BitcoinChain) -> u64 {
        match self {
            Self::Script { script_pubkey, .. } => dust_threshold(chain, script_pubkey),
            Self::External => 0,
        }
    }

    pub(super) fn external_outputs(&self) -> usize {
        match self {
            Self::Script { .. } => 0,
            Self::External => 1,
        }
    }

    // OP_RETURN is placed at output index 1 (after the destination): required by Chainflip's
    // vault-swap scanner, accepted by Thorchain which scans all outputs.
    pub(super) fn outputs(&self, value: u64) -> (Vec<PlanOutput>, Option<u64>) {
        match self {
            Self::Script { script_pubkey, memo_output } => {
                let mut outputs = vec![PlanOutput::new(value, script_pubkey.clone())];
                outputs.extend(memo_output.clone());
                (outputs, None)
            }
            Self::External => (vec![], Some(value)),
        }
    }
}

fn op_return_output(data: &[u8]) -> Result<PlanOutput, SignerError> {
    if data.len() > MAX_OP_RETURN_BYTES {
        return SignerError::invalid_input_err("Bitcoin memo is too large");
    }
    let push = PushBytesBuf::try_from(data.to_vec()).map_err(SignerError::from_display)?;
    Ok(PlanOutput::new(0, Builder::new().push_opcode(OP_RETURN).push_slice(push).into_script()))
}

pub(super) fn dust_threshold(chain: BitcoinChain, script_pubkey: &ScriptBuf) -> u64 {
    if script_pubkey.is_op_return() {
        return 0;
    }
    let threshold = script_pubkey.minimal_non_dust().to_sat();
    match chain {
        BitcoinChain::Doge => threshold.saturating_mul(chain.minimum_byte_fee() as u64),
        BitcoinChain::Bitcoin | BitcoinChain::BitcoinCash | BitcoinChain::Litecoin | BitcoinChain::Dash | BitcoinChain::Zcash => threshold,
    }
}
