use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AgentSetAbstraction {
    pub r#type: String,
    pub abstraction: String,
}

impl AgentSetAbstraction {
    pub fn unified_account() -> Self {
        Self {
            r#type: "agentSetAbstraction".to_string(),
            abstraction: "u".to_string(),
        }
    }
}
