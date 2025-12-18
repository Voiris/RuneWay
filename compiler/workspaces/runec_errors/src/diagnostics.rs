use std::fmt::{Display, Formatter};
use crate::labels::{DiagHelp, DiagLabel, DiagNote};
use crate::message::DiagMessage;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagType {
    WeakWarning,
    Warning,
    Error,
}

impl Display for DiagType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            DiagType::WeakWarning => "\x1b[1;38;2;133;112;66mweak\x1b[0m".fmt(f),
            DiagType::Warning => "\x1b[1;93mwarning\x1b[0m".fmt(f),
            DiagType::Error => "\x1b[1;91merror\x1b[0m".fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct Diagnostic<'a> {
    pub diag_type: DiagType,
    pub code: Option<u16>,
    pub message: DiagMessage<'a>,
    pub labels: Vec<DiagLabel<'a>>,
    pub note: Option<DiagNote<'a>>,
    pub help: Option<DiagHelp<'a>>,
}
