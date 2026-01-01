use std::fmt::{Display, Formatter};
use crate::labels::{DiagHelp, DiagLabel, DiagNote};
use crate::lint::Lint;
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
pub struct Diagnostic<'diag> {
    pub diag_type: DiagType,
    pub code: Option<u16>,
    pub lint_type: Option<Lint<'diag>>,
    pub message: DiagMessage<'diag>,
    pub labels: Vec<DiagLabel<'diag>>,
    pub note: Option<DiagNote<'diag>>,
    pub help: Option<DiagHelp<'diag>>,
}

impl<'diag> Diagnostic<'diag> {
    pub fn new(diag_type: DiagType, code: Option<u16>, message: DiagMessage<'diag>) -> Box<Self> {
        Box::new(Self {
            diag_type,
            code,
            lint_type: None,
            message,
            labels: vec![],
            note: None,
            help: None,
        })
    }

    pub fn weak_warning(message: DiagMessage<'diag>) -> Box<Self> {
        Self::new(DiagType::WeakWarning, None, message)
    }

    pub fn warning(message: DiagMessage<'diag>) -> Box<Self> {
        Self::new(DiagType::Warning, None, message)
    }

    pub fn error(message: DiagMessage<'diag>) -> Box<Self> {
        Self::new(DiagType::Error, None, message)
    }

    pub fn error_with_code(message: DiagMessage<'diag>, code: u16) -> Box<Self> {
        Self::new(DiagType::Error, Some(code), message)
    }

    pub fn add_label(mut self: Box<Self>, label: DiagLabel<'diag>) -> Box<Self> {
        self.labels.push(label);
        self
    }

    pub fn set_help(mut self: Box<Self>, help: DiagHelp<'diag>) -> Box<Self> {
        self.help = Some(help);
        self
    }

    pub fn set_note(mut self: Box<Self>, note: DiagNote<'diag>) -> Box<Self> {
        self.note = Some(note);
        self
    }

    pub fn set_lint_type(mut self: Box<Self>, lint_type: &'diag str) -> Box<Self> {
        self.lint_type = Some(Lint::from_str(lint_type));
        self
    }
}
