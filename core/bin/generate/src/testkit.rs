use std::fs;
use std::path::Path;

use crate::remote_mappers::{Config, Generator};

impl Generator {
    pub fn mock() -> Self {
        let testdata = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
        let yaml = fs::read_to_string(testdata.join("remote_types.yml")).unwrap();
        Self::parse(Config::from_yaml(&yaml), &testdata.join("primitives"))
    }
}
