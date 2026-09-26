use crate::constants::ACCEPT_TERMS_ITEMS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAcceptTermsItem {
    SelfCustody,
    Recovery,
    Responsibility,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTermRow {
    pub item: GemAcceptTermsItem,
    pub is_accepted: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTermsViewState {
    pub rows: Vec<GemTermRow>,
    pub is_accepted: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTermsSession {
    pub accepted: Vec<GemAcceptTermsItem>,
}

#[uniffi::export]
impl GemTermsSession {
    pub fn on_toggle(&self, item: GemAcceptTermsItem) -> Self {
        let accepted = match self.accepted.contains(&item) {
            true => self.accepted.iter().copied().filter(|accepted| *accepted != item).collect(),
            false => self.accepted.iter().copied().chain([item]).collect(),
        };
        Self { accepted }
    }

    pub fn view_state(&self) -> GemTermsViewState {
        let rows: Vec<GemTermRow> = ACCEPT_TERMS_ITEMS
            .iter()
            .map(|item| GemTermRow {
                item: *item,
                is_accepted: self.accepted.contains(item),
            })
            .collect();
        GemTermsViewState {
            is_accepted: rows.iter().all(|row| row.is_accepted),
            rows,
        }
    }
}

#[uniffi::export]
pub fn new_terms_session() -> GemTermsSession {
    GemTermsSession { accepted: Vec::new() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSecurityReminderItem {
    KeepSafe,
    DoNotShare,
    NoRecovery,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terms_are_accepted_only_once_every_one_is_ticked() {
        let session = new_terms_session();
        assert!(!session.view_state().is_accepted);
        assert!(session.view_state().rows.iter().all(|row| !row.is_accepted));

        let all = ACCEPT_TERMS_ITEMS.iter().fold(session, |session, item| session.on_toggle(*item));
        assert!(all.view_state().is_accepted);

        let untick = all.on_toggle(GemAcceptTermsItem::Recovery);
        assert!(!untick.view_state().is_accepted, "unticking a term takes the acceptance back");
        assert_eq!(untick.view_state().rows.iter().filter(|row| row.is_accepted).count(), ACCEPT_TERMS_ITEMS.len() - 1);
    }
}
