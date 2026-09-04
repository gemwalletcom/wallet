use gem_client::Target;

use crate::constants::{PATH_CONFIG, PATH_CONTACT_SET_USER, PATH_MESSAGES, PATH_TOGGLE_TYPING, PATH_UPDATE_LAST_SEEN};

#[derive(Clone, Debug)]
pub enum ChatwootTarget {
    Config,
    SetContact,
    Messages,
    ToggleTyping,
    UpdateLastSeen,
}

impl Target for ChatwootTarget {
    fn path(&self) -> String {
        let path = match self {
            Self::Config => PATH_CONFIG,
            Self::SetContact => PATH_CONTACT_SET_USER,
            Self::Messages => PATH_MESSAGES,
            Self::ToggleTyping => PATH_TOGGLE_TYPING,
            Self::UpdateLastSeen => PATH_UPDATE_LAST_SEEN,
        };
        format!("/api/v1/widget/{path}")
    }
}
