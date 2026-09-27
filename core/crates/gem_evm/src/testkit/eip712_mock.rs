use crate::eip712::{EIP712Domain, EIP712Field, EIP712Message, EIP712Type, EIP712TypedValue, eip712_domain_types};

impl EIP712Domain {
    pub fn mock(chain_id: u64) -> Self {
        Self {
            name: Some("Test".to_string()),
            version: Some("1".to_string()),
            chain_id: Some(chain_id),
            verifying_contract: None,
            salts: None,
        }
    }
}

impl EIP712Message {
    pub fn mock(chain_id: u64) -> Self {
        Self {
            domain: EIP712Domain::mock(chain_id),
            primary_type: "Message".to_string(),
            message: vec![EIP712Field {
                name: "content".to_string(),
                value: EIP712TypedValue::String { value: "Hello".to_string() },
            }],
        }
    }

    pub fn to_json_string(&self) -> String {
        let domain_types: Vec<EIP712Type> = eip712_domain_types().into_iter().filter(|field| field.name != "verifyingContract").collect();
        serde_json::to_string(&serde_json::json!({
            "types": {
                "EIP712Domain": domain_types,
                "Message": [
                    { "name": "content", "type": "string" }
                ]
            },
            "primaryType": self.primary_type,
            "domain": self.domain,
            "message": {
                "content": "Hello"
            }
        }))
        .unwrap()
    }
}

pub fn mock_eip712_json(chain_id: u64) -> String {
    EIP712Message::mock(chain_id).to_json_string()
}
