use crate::runeway::core::errors::{RuneWayError, RuneWayErrorKind};

impl From<std::io::Error> for Box<RuneWayError> {
    fn from(value: std::io::Error) -> Self {
        RuneWayError::new(RuneWayErrorKind::error_with_code("HTTPError"))
            .with_message(format!("{}", value))
    }
}
