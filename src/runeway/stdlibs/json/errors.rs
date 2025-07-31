use crate::runeway::core::errors::{RuneWayError, RuneWayErrorKind};

impl From<serde_json::Error> for Box<RuneWayError> {
    fn from(value: serde_json::Error) -> Self {
        RuneWayError::new(RuneWayErrorKind::error_with_code("JSONParsingError"))
            .with_message(format!("{}", value))
    }
}
