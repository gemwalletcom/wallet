use std::collections::HashMap;

use super::model::{Record, RecordData, RecordName};

impl Record {
    pub fn mock() -> Self {
        Self {
            name: RecordName {
                resolved: "0xf26f5551e96ae5162509b25925fffa7f07b2d652".to_string(),
            },
            data: RecordData {
                chain_addresses: HashMap::from([
                    ("60".to_string(), "0xb43f5153b1c867bf78acb3c35aa9b8ae366415c5".to_string()),
                    ("501".to_string(), "CKAvaYmwqCbg8nZCUCNj6Cvr11HauALtNoGT7WirPoAp".to_string()),
                ]),
            },
        }
    }
}
