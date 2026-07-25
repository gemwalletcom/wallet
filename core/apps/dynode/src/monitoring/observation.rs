use std::time::Duration;

use primitives::NodeStatusState;

use super::switch_reason::CurrentNodeErrorKind;
use crate::config::Url;

#[derive(Debug)]
pub(crate) struct NodeStatusObservation {
    pub(crate) url: Url,
    pub(crate) state: NodeStatusState,
    pub(crate) latency: Duration,
    pub(super) error_kind: CurrentNodeErrorKind,
}

impl NodeStatusObservation {
    pub(crate) fn new(url: Url, state: NodeStatusState, latency: Duration) -> Self {
        Self {
            url,
            state,
            latency,
            error_kind: CurrentNodeErrorKind::Unknown,
        }
    }

    pub(super) fn with_error_kind(self, error_kind: CurrentNodeErrorKind) -> Self {
        Self { error_kind, ..self }
    }
}
