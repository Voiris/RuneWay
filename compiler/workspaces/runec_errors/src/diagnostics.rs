use crate::labels::{DiagHelp, DiagLabel, DiagNote};
use crate::message::DiagMessage;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagType {
    WeakWarning,
    Warning,
    Error,
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
