use crate::services::error::GemServiceError;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemLoadState {
    NoData,
    Loading,
    Data,
    Error { error: GemServiceError },
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

impl GemLoadState {
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
    fn test_a_failed_refresh_keeps_the_rows_already_stored() {
        let failed = || Err(GemServiceError::Gateway { msg: "offline".to_string() });

        assert_eq!(GemLoadState::refreshed(Ok(()), false), GemLoadState::Data);
        assert_eq!(GemLoadState::refreshed(failed(), true), GemLoadState::Data);
        assert!(matches!(GemLoadState::refreshed(failed(), false), GemLoadState::Error { .. }));
    }
}
