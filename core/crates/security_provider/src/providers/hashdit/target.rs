use std::collections::HashMap;

use gem_client::{CONTENT_TYPE, Target, build_path_with_query};

#[derive(Clone, Debug)]
pub enum HashDitTarget {
    Detect { business: &'static str },
}

impl Target for HashDitTarget {
    fn path(&self) -> String {
        match self {
            Self::Detect { business } => build_path_with_query("/security-api/public/app/v1/detect", &[("business", business)]),
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        HashMap::from([(CONTENT_TYPE.to_string(), "application/json;charset=UTF-8".to_string())])
    }
}
