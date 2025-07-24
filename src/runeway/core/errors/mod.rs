mod color_generator;
mod impls;

use std::error::Error;
use std::fmt;
use std::ops::Range;
use ariadne::{Color, Label, Report, ReportKind, Source};

#[derive(Debug, Clone)]
pub struct RuneWayError {
    pub message: Option<String>,
    pub labels: Vec<RuneWayErrorLabel>,
    pub additions: Vec<RuneWayAddition>,
    pub kind: RuneWayErrorKind,
    pub filename: Option<String>,
    pub code: Option<String>,
}

impl RuneWayError {
    pub fn new(
        kind: RuneWayErrorKind
    ) -> Self {
        Self {
            kind,
            labels: Vec::new(),
            additions: Vec::new(),
            message: None,
            filename: None,
            code: None,
        }
    }

    pub fn with_message<M: Into<String>>(mut self, message: M) -> Self {
        self.message = Some(message.into());

        self
    }

    pub fn with_label<L: Into<String>>(mut self, label: L, span: &Range<usize>, color: Option<Color>) -> Self {
        self.labels.push(RuneWayErrorLabel {
            span: span.clone(),
            label: label.into(),
            color
        });

        self
    }

    pub fn with_help<L: Into<String>>(mut self, help: L) -> Self {
        self.additions.push(RuneWayAddition::Help(help.into()));

        self
    }

    pub fn with_note<L: Into<String>>(mut self, note: L) -> Self {
        self.additions.push(RuneWayAddition::Note(note.into()));

        self
    }

    pub fn with_code_base(mut self, filename: impl ToString, code: impl ToString) -> Self {
        if self.filename.is_none() || self.code.is_none() {
            self.filename = Some(filename.to_string());
            self.code = Some(code.to_string());
        }

        self
    }

    pub fn report(&self) {
        let (source_id, source_code): (String, String) = match (self.filename.clone(), self.code.clone()) {
            (Some(filename), Some(code)) => {
                (filename, code)
            }
            _ => panic!("Internal: Cannot report error without code base"),
        };

        let source_id = source_id.as_str();
        let source_code = source_code.as_str();

        let main_range = self.find_error_start().unwrap_or_else(|| {
            let size = source_code.len();

            size..size
        });

        let mut report =
            Report::build(self.kind.report_kind(), (source_id, main_range));

        if let Some(message) = self.message.as_ref() {
            report = report.with_message(message)
        }

        for label in self.labels.iter() {
            report = report.with_label(
                Label::new((source_id, label.span.clone()))
                    .with_message(&label.label)
                    .with_color(label.color()),
            )
        }

        for addition in self.additions.iter() {
            match addition {
                RuneWayAddition::Help(help) => report = report.with_help(help),
                RuneWayAddition::Note(note) => report = report.with_note(note),
            }
        }

        println!();
        report.finish().print((source_id, Source::from(source_code))).unwrap();
    }

    fn find_error_start(&self) -> Option<Range<usize>> {
        let start = self.labels.iter()
            .map(|label| label.span.start)
            .min();
        match start {
            Some(start) => Some(start..start),
            None => None
        }
    }
}

impl fmt::Display for RuneWayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind.report_kind(), self.message.as_ref().unwrap())
    }
}

#[derive(Debug, Clone)]
pub enum RuneWayErrorKind {
    Syntax,
    Warning,
    Type,
    Runtime(Option<String>),
}

impl RuneWayErrorKind {
    pub fn report_kind(&self) -> ReportKind {
        match self {
            Self::Syntax => ReportKind::Custom("SyntaxError", Color::Red),
            Self::Warning => ReportKind::Warning,
            Self::Type => ReportKind::Custom("TypeError", Color::Yellow),
            Self::Runtime(error) =>
                ReportKind::Custom(error.as_deref().unwrap_or("RuntimeError"), Color::BrightCyan),
        }
    }
}

#[derive(Debug, Clone)]
struct RuneWayErrorLabel {
    pub span: Range<usize>,
    pub label: String,
    pub color: Option<Color>,
}

impl RuneWayErrorLabel {
    fn color(&self) -> Color {
        self.color.unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
enum RuneWayAddition {
    Note(String),
    Help(String),
}

pub type RWResult<T> = Result<T, RuneWayError>;
