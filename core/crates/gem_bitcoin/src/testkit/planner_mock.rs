use bitcoin::{
    Amount, OutPoint, ScriptBuf,
    blockdata::{opcodes::all::OP_RETURN, script::Builder},
    script::PushBytesBuf,
};

use crate::signer::{
    PlanInput, PlanOutput,
    address::{UnlockingScript, script_for_public_key_hash},
};

impl PlanInput {
    pub fn mock_with_unlocking_script(unlocking_script: UnlockingScript) -> Self {
        Self {
            previous_output: OutPoint::null(),
            value: Amount::from_sat(50_000),
            script_pubkey: ScriptBuf::new(),
            unlocking_script,
            sequence: u32::MAX,
        }
    }
}

impl PlanOutput {
    pub fn mock_with_unlocking_script(unlocking_script: UnlockingScript, value: u64) -> Self {
        Self::new(value, script_for_public_key_hash(unlocking_script, [0u8; 20]))
    }
}

pub fn op_return_script(bytes: usize) -> ScriptBuf {
    let push = PushBytesBuf::try_from(vec![0u8; bytes]).unwrap();
    Builder::new().push_opcode(OP_RETURN).push_slice(push).into_script()
}
