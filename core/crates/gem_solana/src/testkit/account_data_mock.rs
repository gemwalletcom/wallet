use crate::{
    SYSTEM_PROGRAM_ID,
    models::{AccountData, ValueData},
};

impl AccountData {
    pub(crate) fn mock() -> Self {
        ValueData {
            data: vec![],
            owner: SYSTEM_PROGRAM_ID.to_string(),
        }
    }
}
