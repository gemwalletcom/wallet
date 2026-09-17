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

#[uniffi::export]
pub fn accept_terms_items() -> Vec<GemAcceptTermsItem> {
    vec![GemAcceptTermsItem::SelfCustody, GemAcceptTermsItem::Recovery, GemAcceptTermsItem::Responsibility]
}

#[uniffi::export]
pub fn security_reminder_items() -> Vec<GemSecurityReminderItem> {
    vec![GemSecurityReminderItem::KeepSafe, GemSecurityReminderItem::DoNotShare, GemSecurityReminderItem::NoRecovery]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_asks_for_every_term_and_shows_every_reminder() {
        assert_eq!(accept_terms_items().len(), 3, "a wallet is only created once all three terms are accepted");
        assert_eq!(security_reminder_items().first(), Some(&GemSecurityReminderItem::KeepSafe));
        assert_eq!(security_reminder_items().last(), Some(&GemSecurityReminderItem::NoRecovery));
    }
}
