use crate::models::UsernameRow;
use crate::sql_types::UsernameStatus;

impl UsernameRow {
    pub fn mock(username: &str) -> Self {
        Self {
            username: username.to_string(),
            wallet_id: 1,
            status: UsernameStatus::Unverified,
        }
    }
}
