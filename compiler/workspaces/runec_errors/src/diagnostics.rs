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

impl<'a> Diagnostic<'a> {
    pub fn new(diag_type: DiagType, code: Option<u16>, message: DiagMessage<'a>) -> Self {
        Self {
            diag_type,
            code,
            message,
            labels: vec![],
            note: None,
            help: None,
        }
    }

    pub fn weak_warning(message: DiagMessage<'a>) -> Self {
        Self::new(DiagType::WeakWarning, None, message)
    }

    pub fn warning(message: DiagMessage<'a>) -> Self {
        Self::new(DiagType::Warning, None, message)
    }

    pub fn error(message: DiagMessage<'a>) -> Self {
        Self::new(DiagType::Error, None, message)
    }

    pub fn error_with_code(message: DiagMessage<'a>, code: u16) -> Self {
        Self::new(DiagType::Error, Some(code), message)
    }

    pub fn add_label(mut self, label: DiagLabel<'a>) -> Self {
        self.labels.push(label);
        self
    }

    pub fn add_help(mut self, help: DiagHelp<'a>) -> Self {
        self.help = Some(help);
        self
    }

    pub fn add_note(mut self, note: DiagNote<'a>) -> Self {
        self.note = Some(note);
        self
    }
}
