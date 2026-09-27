use std::fs;
use std::path::Path;

use crate::remote_mappers::{Config, Generator};

impl Generator {
    pub fn mock() -> Self {
        let testdata = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
        let yaml = fs::read_to_string(testdata.join("remote_types.yml")).unwrap();
        Self::parse(Config::from_yaml(&yaml), &testdata.join("primitives"), &testdata.join("gemstone"))
    }
}

/// `testdata/` holds a `remote_types.yml`, one primitives source per generator feature under
/// `primitives/` and, under `expected/`, the exact files the generator must
/// write for them. Run with `UPDATE_GOLDEN=1` to rewrite the expected files after a deliberate
/// change, and read the diff before committing it.
pub fn expect_generated(name: &str, actual: String) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata").join("expected").join(name);
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        fs::write(&path, &actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} is missing; run the tests once with UPDATE_GOLDEN=1", path.display()));
    assert_eq!(actual, expected, "{name} no longer matches testdata/expected/{name}; rerun with UPDATE_GOLDEN=1 once the diff is intended");
}
