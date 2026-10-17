use num_bigint::BigUint;

use crate::{
    address::Address,
    constants::JETTON_TRANSFER_OPCODE,
    tvm::{Cell, CellArc, CellBuilder, TvmError},
};

pub struct JettonTransfer {
    pub query_id: u64,
    pub amount: BigUint,
    pub destination: Address,
    pub response_destination: Option<Address>,
    pub custom_payload: Option<CellArc>,
    pub forward_ton_amount: BigUint,
    pub forward_payload: Option<CellArc>,
}

impl JettonTransfer {
    pub fn new(amount: BigUint, destination: Address, response_destination: Option<Address>, forward_ton_amount: BigUint, forward_payload: Option<CellArc>) -> Self {
        Self {
            query_id: 0,
            amount,
            destination,
            response_destination,
            custom_payload: None,
            forward_ton_amount,
            forward_payload,
        }
    }

    pub fn to_cell(&self) -> Result<Cell, TvmError> {
        CellBuilder::new()
            .store_u32(32, JETTON_TRANSFER_OPCODE)?
            .store_u64(64, self.query_id)?
            .store_coins(&self.amount)?
            .store_address(&self.destination)?
            .store_maybe_address(self.response_destination.as_ref())?
            .store_maybe_reference(self.custom_payload.as_ref())?
            .store_coins(&self.forward_ton_amount)?
            .store_maybe_reference(self.forward_payload.as_ref())?
            .end_cell()
    }
}
