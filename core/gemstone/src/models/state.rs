use crate::services::error::GemServiceError;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemLoadState {
    NoData,
    Loading,
    Data,
    Error { error: GemServiceError },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRefreshResult {
    pub state: GemLoadState,
    pub toast: Option<GemServiceError>,
}

impl GemRefreshResult {
    pub fn new(synced: Result<(), GemServiceError>, shows_value: bool) -> Self {
        Self {
            toast: synced.as_ref().err().filter(|_| shows_value).cloned(),
            state: GemLoadState::refreshed(synced, shows_value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GemLoad<T> {
    pub state: GemLoadState,
    pub value: T,
}

impl<T: Clone + Default> GemLoad<T> {
    pub fn loading() -> Self {
        Self {
            state: GemLoadState::Loading,
            value: T::default(),
        }
    }

    pub fn data(&self, value: Result<T, GemServiceError>) -> Self {
        match (value, &self.state) {
            (Ok(value), _) => Self { state: GemLoadState::Data, value },
            (Err(_), GemLoadState::Data) => self.clone(),
            (Err(error), GemLoadState::NoData | GemLoadState::Loading | GemLoadState::Error { .. }) => Self {
                state: GemLoadState::Error { error },
                value: T::default(),
            },
        }
    }
}

#[uniffi::export]
pub fn load_error(state: GemLoadState, has_rows: bool) -> Option<GemServiceError> {
    match state {
        GemLoadState::Error { error } if !has_rows => Some(error),
        _ => None,
    }
}

impl GemLoadState {
    pub fn of<T>(value: &Result<T, GemServiceError>) -> Self {
        match value {
            Ok(_) => Self::Data,
            Err(error) => Self::Error { error: error.clone() },
        }
    }

    pub fn into_result<T>(self, value: T) -> Result<T, GemServiceError> {
        match self {
            Self::Error { error } => Err(error),
            Self::NoData | Self::Loading | Self::Data => Ok(value),
        }
    }

    pub fn refreshed(synced: Result<(), GemServiceError>, shows_value: bool) -> Self {
        let shown = GemLoad {
            state: match shows_value {
                true => Self::Data,
                false => Self::NoData,
            },
            value: (),
        };
        shown.data(synced).state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_keeps_the_value_the_screen_already_shows() {
        let error = GemServiceError::Gateway { msg: "offline".to_string() };
        let shown = GemLoad::loading().data(Ok(vec!["row".to_string()]));

        assert_eq!(
            shown,
            GemLoad {
                state: GemLoadState::Data,
                value: vec!["row".to_string()]
            }
        );
        assert_eq!(shown.data(Err(error.clone())), shown, "a failed refresh keeps what the screen shows");
        assert_eq!(
            GemLoad::<Vec<String>>::loading().data(Err(error.clone())),
            GemLoad {
                state: GemLoadState::Error { error: error.clone() },
                value: Vec::new()
            }
        );
    }

    #[test]
    fn test_only_a_screen_with_nothing_to_show_reports_its_failure() {
        let error = GemServiceError::Gateway { msg: "offline".to_string() };

        assert_eq!(load_error(GemLoadState::Error { error: error.clone() }, false), Some(error.clone()));
        assert_eq!(load_error(GemLoadState::Error { error }, true), None, "rows on screen stand in for the error");
        assert_eq!(load_error(GemLoadState::Loading, false), None);
        assert_eq!(load_error(GemLoadState::Data, false), None);
    }

    #[test]
    fn test_a_failed_refresh_keeps_the_rows_already_stored() {
        let failed = || Err(GemServiceError::Gateway { msg: "offline".to_string() });

        assert_eq!(GemLoadState::refreshed(Ok(()), false), GemLoadState::Data);
        assert_eq!(GemLoadState::refreshed(failed(), true), GemLoadState::Data);
        assert!(matches!(GemLoadState::refreshed(failed(), false), GemLoadState::Error { .. }));
    }

    #[test]
    fn test_a_failed_refresh_over_shown_rows_is_a_toast() {
        let error = GemServiceError::Gateway { msg: "offline".to_string() };

        assert_eq!(
            GemRefreshResult::new(Err(error.clone()), true),
            GemRefreshResult {
                state: GemLoadState::Data,
                toast: Some(error.clone())
            }
        );
        assert_eq!(
            GemRefreshResult::new(Err(error.clone()), false),
            GemRefreshResult {
                state: GemLoadState::Error { error },
                toast: None
            },
            "an empty screen shows the error row instead"
        );
        assert_eq!(GemRefreshResult::new(Ok(()), true).toast, None);
    }
}
