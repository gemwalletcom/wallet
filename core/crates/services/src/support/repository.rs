use async_trait::async_trait;
use primitives::Device;
use storage::{Database, DatabaseError, DevicesRepository, SupportSessionsRepository};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn get_device(&self, device_id: String) -> Result<Option<Device>, DatabaseError>;
    async fn get_session_token(&self, device_id: i32) -> Result<Option<String>, DatabaseError>;
    async fn set_session_token(&self, device_id: i32, auth_token: String) -> Result<usize, DatabaseError>;
}

pub(crate) struct PostgresRepository {
    database: Database,
}

impl PostgresRepository {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn get_device(&self, device_id: String) -> Result<Option<Device>, DatabaseError> {
        self.database
            .run(move |client| match client.get_device(&device_id) {
                Ok(device) => Ok(Some(device)),
                Err(error) if error.is_not_found() => Ok(None),
                Err(error) => Err(error),
            })
            .await
    }

    async fn get_session_token(&self, device_id: i32) -> Result<Option<String>, DatabaseError> {
        self.database.run(move |client| client.get_support_session_token(device_id)).await
    }

    async fn set_session_token(&self, device_id: i32, auth_token: String) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.set_support_session_token(device_id, &auth_token)).await
    }
}
