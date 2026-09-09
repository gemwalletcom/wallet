use crate::swap::ApprovalData;

#[derive(Debug, Clone)]
pub struct ContractCallData {
    pub contract_address: String,
    pub call_data: String,
    pub approval: Option<ApprovalData>,
    pub gas_limit: Option<String>,
}
