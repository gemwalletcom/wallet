use crate::models::EpochInfo;

impl EpochInfo {
    pub fn mock(slot_index: u64) -> Self {
        EpochInfo {
            epoch: 200,
            slot_index,
            slots_in_epoch: 432000,
        }
    }
}
