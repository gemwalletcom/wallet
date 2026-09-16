use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum SolanaNftStandard {
    NonFungible,
    ProgrammableNonFungible { rule_set: Option<String> },
    Core { collection: Option<String> },
}
