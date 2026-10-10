use diesel::prelude::*;

use crate::{DatabaseClient, DatabaseError};

pub trait ScanWebsitesRepository {
    fn get_scan_websites(&mut self, hosts: Vec<String>) -> Result<Vec<String>, DatabaseError>;
}

impl ScanWebsitesRepository for DatabaseClient {
    fn get_scan_websites(&mut self, hosts: Vec<String>) -> Result<Vec<String>, DatabaseError> {
        use crate::schema::scan_websites::dsl::*;
        Ok(scan_websites.filter(host.eq_any(hosts)).filter(is_fraudulent.eq(true)).select(host).load(&mut self.connection)?)
    }
}
