use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveOption<T> {
    pub vec: Vec<T>,
}
