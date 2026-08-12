use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub runtime_config: RuntimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub min_gas_purchase_price: u64,
    pub transaction_costs: TransactionCosts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCosts {
    pub action_receipt_creation_config: ActionCost,
    pub action_creation_config: ActionCreationCosts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCreationCosts {
    pub create_account_cost: ActionCost,
    pub function_call_cost: ActionCost,
    pub transfer_cost: ActionCost,
    pub add_key_cost: AddKeyCost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddKeyCost {
    pub full_access_cost: ActionCost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCost {
    pub execution: u64,
    pub send_sir: u64,
    pub send_not_sir: u64,
}

impl ActionCost {
    pub fn send_gas(&self, sender_is_receiver: bool) -> u64 {
        if sender_is_receiver { self.send_sir } else { self.send_not_sir }
    }
}
