use std::error::Error;
use std::num::{ParseFloatError, ParseIntError};
use crate::runeway::core::errors::{RuneWayError, RuneWayErrorKind};

impl From<Box<dyn Error>> for RuneWayError {
    fn from(e: Box<dyn Error>) -> Self {
        RuneWayError::new(RuneWayErrorKind::Runtime(Some("JSONParsingError".to_string())))
            .with_message(format!("{}", e))
    }
}

impl From<ParseIntError> for RuneWayError {
    fn from(e: ParseIntError) -> RuneWayError {
        RuneWayError::new(RuneWayErrorKind::Runtime(Some("JSONParsingError".to_string())))
            .with_message(format!("{}", e))
    }
}

impl From<ParseFloatError> for RuneWayError {
    fn from(e: ParseFloatError) -> RuneWayError {
        RuneWayError::new(RuneWayErrorKind::Runtime(Some("JSONParsingError".to_string())))
            .with_message(format!("{}", e))
    }
}
