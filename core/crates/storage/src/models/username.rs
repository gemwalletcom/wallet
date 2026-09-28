use crate::sql_types::UsernameStatus;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::usernames)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct UsernameRow {
    pub username: String,
    pub wallet_id: i32,
    pub status: UsernameStatus,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::usernames)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewUsernameRow {
    pub username: String,
    pub wallet_id: i32,
    pub status: UsernameStatus,
}
