use crate::database::support_sessions::SupportSessionsStore;
use crate::models::{NewSupportSessionRow, SupportSessionRow};
use crate::{DatabaseClient, DatabaseError};

pub trait SupportSessionsRepository {
    fn get_support_session(&mut self, device_id: i32) -> Result<Option<SupportSessionRow>, DatabaseError>;
    fn set_support_session(&mut self, value: NewSupportSessionRow) -> Result<SupportSessionRow, DatabaseError>;
}

impl SupportSessionsRepository for DatabaseClient {
    fn get_support_session(&mut self, device_id: i32) -> Result<Option<SupportSessionRow>, DatabaseError> {
        Ok(SupportSessionsStore::get_support_session(self, device_id)?)
    }

    fn set_support_session(&mut self, value: NewSupportSessionRow) -> Result<SupportSessionRow, DatabaseError> {
        Ok(SupportSessionsStore::set_support_session(self, value)?)
    }
}
