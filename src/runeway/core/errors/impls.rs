use crate::runeway::core::errors::{RuneWayError, RuneWayErrorKind};
use std::error::Error;
use std::num::{ParseFloatError, ParseIntError};

impl From<Box<dyn Error>> for Box<RuneWayError> {
    fn from(e: Box<dyn Error>) -> Self {
        RuneWayError::new(RuneWayErrorKind::error_with_code("JSONParsingError"))
            .with_message(format!("{}", e))
    }
}

impl From<ParseIntError> for Box<RuneWayError> {
    fn from(e: ParseIntError) -> Self {
        RuneWayError::new(RuneWayErrorKind::error_with_code("JSONParsingError"))
            .with_message(format!("{}", e))
    }
}

impl From<ParseFloatError> for Box<RuneWayError> {
    fn from(e: ParseFloatError) -> Self {
        RuneWayError::new(RuneWayErrorKind::error_with_code("JSONParsingError"))
            .with_message(format!("{}", e))
    }
}

impl From<String> for Box<RuneWayError> {
    fn from(message: String) -> Self {
        RuneWayError::new(RuneWayErrorKind::runtime_error()).with_message(message)
    }
}

impl From<(&'static str, String)> for Box<RuneWayError> {
    fn from((kind, message): (&'static str, String)) -> Self {
        RuneWayError::new(RuneWayErrorKind::error_with_code(kind)).with_message(message)
    }
}
