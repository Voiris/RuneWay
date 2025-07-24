use crate::runeway::core::errors::{RuneWayError, RuneWayErrorKind};

impl From<reqwest::Error> for RuneWayError {
    fn from(value: reqwest::Error) -> Self {
        RuneWayError::new(RuneWayErrorKind::Runtime(Some("HTTPError".to_string())))
            .with_message(format!("{}", value))
    }
}
