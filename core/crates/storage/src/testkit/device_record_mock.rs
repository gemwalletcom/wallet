use chrono::NaiveDateTime;
use primitives::Device;

use crate::repositories::devices_repository::DeviceRecord;

impl DeviceRecord {
    pub fn mock() -> Self {
        Self {
            id: 1,
            device: Device::mock(),
            created_at: NaiveDateTime::default(),
        }
    }
}
