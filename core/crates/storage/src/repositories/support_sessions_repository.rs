use diesel::prelude::*;
use diesel::upsert::excluded;

use crate::models::{NewSupportSessionRow, SupportSessionRow};
use crate::{DatabaseClient, DatabaseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportSessionRecord {
    pub auth_token: String,
    pub blocked: bool,
}

pub trait SupportSessionsRepository {
    fn get_support_session(&mut self, device_id: i32) -> Result<Option<SupportSessionRecord>, DatabaseError>;
    fn set_support_session_token(&mut self, device_id: i32, auth_token: &str) -> Result<usize, DatabaseError>;
    fn set_support_session_blocked(&mut self, device_identifier: &str, blocked: bool) -> Result<usize, DatabaseError>;
}

impl SupportSessionsRepository for DatabaseClient {
    fn get_support_session(&mut self, device_id_value: i32) -> Result<Option<SupportSessionRecord>, DatabaseError> {
        use crate::schema::support_sessions::dsl::*;
        let row = support_sessions.filter(device_id.eq(device_id_value)).select(SupportSessionRow::as_select()).first(&mut self.connection).optional()?;
        Ok(row.map(|row| SupportSessionRecord {
            auth_token: row.auth_token,
            blocked: row.blocked,
        }))
    }

    fn set_support_session_token(&mut self, device_id_value: i32, auth_token_value: &str) -> Result<usize, DatabaseError> {
        use crate::schema::support_sessions::dsl::*;
        let value = NewSupportSessionRow::new(device_id_value, auth_token_value);
        Ok(diesel::insert_into(support_sessions)
            .values(&value)
            .on_conflict(device_id)
            .do_update()
            .set(auth_token.eq(excluded(auth_token)))
            .execute(&mut self.connection)?)
    }

    fn set_support_session_blocked(&mut self, device_identifier: &str, blocked_value: bool) -> Result<usize, DatabaseError> {
        use crate::schema::{devices, support_sessions};
        let device = devices::table.filter(devices::identifier.eq(device_identifier)).select(devices::id);
        Ok(diesel::update(support_sessions::table.filter(support_sessions::device_id.eq_any(device)))
            .set(support_sessions::blocked.eq(blocked_value))
            .execute(&mut self.connection)?)
    }
}
