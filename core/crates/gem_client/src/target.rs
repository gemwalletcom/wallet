use std::collections::HashMap;

pub trait Target {
    fn path(&self) -> String;

    fn headers(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

impl Target for &str {
    fn path(&self) -> String {
        self.to_string()
    }
}

impl Target for &String {
    fn path(&self) -> String {
        self.to_string()
    }
}
