use bitcoin::{Amount, OutPoint, ScriptBuf};

use crate::signer::{address::UnlockingScript, encoding::varint_len};

#[derive(Debug, Clone)]
pub struct PlanInput {
    pub previous_output: OutPoint,
    pub value: Amount,
    pub script_pubkey: ScriptBuf,
    pub unlocking_script: UnlockingScript,
    pub sequence: u32,
}

#[derive(Debug, Clone)]
pub struct PlanOutput {
    pub value: Amount,
    pub script_pubkey: ScriptBuf,
}

impl PlanOutput {
    pub fn new(value: u64, script_pubkey: ScriptBuf) -> Self {
        Self {
            value: Amount::from_sat(value),
            script_pubkey,
        }
    }

    pub fn serialized_len(&self) -> u64 {
        8 + varint_len(self.script_pubkey.len()) as u64 + self.script_pubkey.len() as u64
    }
}

#[derive(Debug, Clone)]
pub struct SpendPlan {
    pub inputs: Vec<PlanInput>,
    pub outputs: Vec<PlanOutput>,
    pub external_value: Option<u64>,
    pub fee: u64,
}
