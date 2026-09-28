use std::env;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, TestCustomizer};

use crate::Database;

impl Database {
    pub fn mock() -> Self {
        let manager = ConnectionManager::<PgConnection>::new(env::var("DATABASE_URL").unwrap());
        Self(Pool::builder().max_size(1).connection_customizer(Box::new(TestCustomizer)).build(manager).unwrap())
    }
}
