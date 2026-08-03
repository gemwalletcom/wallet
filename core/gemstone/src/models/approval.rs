pub use primitives::ApprovalData;

pub type GemApprovalData = ApprovalData;

#[uniffi::remote(Record)]
pub struct GemApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
    pub is_unlimited: bool,
}
