#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAcceptTermsItem {
    SelfCustody,
    Recovery,
    Responsibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSecurityReminderItem {
    KeepSafe,
    DoNotShare,
    NoRecovery,
}
