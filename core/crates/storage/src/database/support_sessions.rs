use crate::{DatabaseClient, models::*};
use diesel::prelude::*;
use diesel::upsert::excluded;

pub trait SupportSessionsStore {
    fn get_support_session(&mut self, device_id: i32) -> Result<Option<SupportSessionRow>, diesel::result::Error>;
    fn set_support_session(&mut self, value: NewSupportSessionRow) -> Result<SupportSessionRow, diesel::result::Error>;
}

impl SupportSessionsStore for DatabaseClient {
    fn get_support_session(&mut self, device_id_value: i32) -> Result<Option<SupportSessionRow>, diesel::result::Error> {
        use crate::schema::support_sessions::dsl::*;

        support_sessions
            .filter(device_id.eq(device_id_value))
            .select(SupportSessionRow::as_select())
            .first(&mut self.connection)
            .optional()
    }

    fn set_support_session(&mut self, value: NewSupportSessionRow) -> Result<SupportSessionRow, diesel::result::Error> {
        use crate::schema::support_sessions::dsl::*;

        diesel::insert_into(support_sessions)
            .values(&value)
            .on_conflict(device_id)
            .do_update()
            .set(auth_token.eq(excluded(auth_token)))
            .returning(SupportSessionRow::as_returning())
            .get_result(&mut self.connection)
    }
}
