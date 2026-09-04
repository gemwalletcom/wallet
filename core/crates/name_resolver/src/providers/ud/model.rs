use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ResolveDomain {
    pub records: HashMap<String, String>,
}
