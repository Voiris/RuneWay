use crate::runeway::core::errors::{RuneWayError, RuneWayErrorKind};

impl From<reqwest::Error> for Box<RuneWayError> {
    fn from(value: reqwest::Error) -> Self {
        RuneWayError::new(RuneWayErrorKind::error_with_code("HTTPError"))
            .with_message(format!("{}", value))
    }
}
