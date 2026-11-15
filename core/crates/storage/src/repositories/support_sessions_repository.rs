use diesel::prelude::*;
use diesel::upsert::excluded;

use crate::models::{NewSupportSessionRow, SupportSessionRow};
use crate::{DatabaseClient, DatabaseError};

pub trait SupportSessionsRepository {
    fn get_support_session_token(&mut self, device_id: i32) -> Result<Option<String>, DatabaseError>;
    fn set_support_session_token(&mut self, device_id: i32, auth_token: &str) -> Result<usize, DatabaseError>;
}

impl SupportSessionsRepository for DatabaseClient {
    fn get_support_session_token(&mut self, device_id_value: i32) -> Result<Option<String>, DatabaseError> {
        use crate::schema::support_sessions::dsl::*;
        let row = support_sessions.filter(device_id.eq(device_id_value)).select(SupportSessionRow::as_select()).first(&mut self.connection).optional()?;
        Ok(row.map(|row| row.auth_token))
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
}
