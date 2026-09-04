use std::collections::HashMap;

use gem_client::{CONTENT_TYPE, ContentType, Target, X_CACHE_TTL};

use crate::models::info::InfoRequest;

const INFO_CACHE_TTL_SECS: u64 = 3600;

#[derive(Clone, Debug)]
pub enum HyperCoreTarget {
    Info { request: InfoRequest },
    Exchange,
}

impl Target for HyperCoreTarget {
    fn path(&self) -> String {
        match self {
            Self::Info { .. } => "/info".to_string(),
            Self::Exchange => "/exchange".to_string(),
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        match self {
            Self::Info {
                request: InfoRequest::SpotMeta | InfoRequest::UserAbstraction { .. },
            } => HashMap::from([
                (CONTENT_TYPE.to_string(), ContentType::ApplicationJson.as_str().to_string()),
                (X_CACHE_TTL.to_string(), INFO_CACHE_TTL_SECS.to_string()),
            ]),
            _ => HashMap::new(),
        }
    }
}
