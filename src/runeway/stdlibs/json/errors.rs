use crate::runeway::core::errors::{RuneWayError, RuneWayErrorKind};

impl From<serde_json::Error> for RuneWayError {
    fn from(value: serde_json::Error) -> Self {
        RuneWayError::new(RuneWayErrorKind::Runtime(Some("JSONParsingError".to_string())))
            .with_message(format!("{}", value))
    }
}
