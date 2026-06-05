use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::support_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SupportSessionRow {
    pub device_id: i32,
    pub auth_token: String,
    pub updated_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::support_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSupportSessionRow {
    pub device_id: i32,
    pub auth_token: String,
}

impl NewSupportSessionRow {
    pub fn new(device_id: i32, auth_token: impl Into<String>) -> Self {
        Self {
            device_id,
            auth_token: auth_token.into(),
        }
    }
}
