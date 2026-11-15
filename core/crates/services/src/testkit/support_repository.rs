use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use primitives::Device;
use storage::{DatabaseError, DeviceRecord, SupportSessionRecord};

use crate::support::repository::Repository;

#[derive(Default)]
pub(crate) struct MemorySupportRepository {
    devices: Mutex<Vec<DeviceRecord>>,
    sessions: Mutex<HashMap<i32, SupportSessionRecord>>,
}

impl MemorySupportRepository {
    pub(crate) fn with_session(device: DeviceRecord, session: SupportSessionRecord) -> Self {
        let repository = Self::default();
        repository.sessions.lock().unwrap().insert(device.id, session);
        repository.devices.lock().unwrap().push(device);
        repository
    }

    pub(crate) fn session(&self, device_id: i32) -> Option<SupportSessionRecord> {
        self.sessions.lock().unwrap().get(&device_id).cloned()
    }
}

#[async_trait]
impl Repository for MemorySupportRepository {
    async fn get_device(&self, device_id: String) -> Result<Option<Device>, DatabaseError> {
        Ok(self.devices.lock().unwrap().iter().find(|record| record.device.id == device_id).map(|record| record.device.clone()))
    }

    async fn get_session(&self, device_id: i32) -> Result<Option<SupportSessionRecord>, DatabaseError> {
        Ok(self.session(device_id))
    }

    async fn set_session_token(&self, device_id: i32, auth_token: String) -> Result<usize, DatabaseError> {
        let mut sessions = self.sessions.lock().unwrap();
        let blocked = sessions.get(&device_id).is_some_and(|session| session.blocked);
        sessions.insert(device_id, SupportSessionRecord { auth_token, blocked });
        Ok(1)
    }

    async fn set_session_blocked(&self, device_identifier: String, blocked: bool) -> Result<usize, DatabaseError> {
        let Some(device_id) = self.devices.lock().unwrap().iter().find(|record| record.device.id == device_identifier).map(|record| record.id) else {
            return Ok(0);
        };
        let mut sessions = self.sessions.lock().unwrap();
        let Some(session) = sessions.get_mut(&device_id) else {
            return Ok(0);
        };
        session.blocked = blocked;
        Ok(1)
    }
}
