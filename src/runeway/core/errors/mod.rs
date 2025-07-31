mod impls;

use codespan_reporting::files::Files;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label, LabelStyle, Severity},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use colored::*;
use std::fmt;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct RuneWayError {
    message: Option<String>,
    labels: Vec<RuneWayErrorLabel>,
    additions: Vec<RuneWayAddition>,
    kind: RuneWayErrorKind,
    sources: Option<Vec<(String, String)>>,
}

impl RuneWayError {
    pub fn new(kind: RuneWayErrorKind) -> Box<Self> {
        Box::new(Self {
            kind,
            labels: Vec::new(),
            additions: Vec::new(),
            message: None,
            sources: None,
        })
    }

    pub fn with_message<M: Into<String>>(mut self, message: M) -> Box<Self> {
        self.message = Some(message.into());
        Box::new(self)
    }

    pub fn with_label<L: Into<String>>(
        mut self,
        label: L,
        span: &Range<usize>,
        source_id: &String,
    ) -> Box<Self> {
        self.inner_with_label(label, span, source_id, LabelStyle::Primary)
    }

    pub fn with_secondary_label<L: Into<String>>(
        mut self,
        label: L,
        span: &Range<usize>,
        source_id: &String,
    ) -> Box<Self> {
        self.inner_with_label(label, span, source_id, LabelStyle::Secondary)
    }

    fn inner_with_label<L: Into<String>>(
        mut self,
        label: L,
        span: &Range<usize>,
        source_id: &String,
        label_style: LabelStyle,
    ) -> Box<Self> {
        self.labels.push(RuneWayErrorLabel {
            span: span.clone(),
            label: label.into(),
            source_id: source_id.clone(),
            label_style,
        });
        Box::new(self)
    }

    pub fn with_help<L: Into<String>>(mut self, help: L) -> Box<Self> {
        self.additions.push(RuneWayAddition::Help(help.into()));
        Box::new(self)
    }

    pub fn with_note<L: Into<String>>(mut self, note: L) -> Box<Self> {
        self.additions.push(RuneWayAddition::Note(note.into()));
        Box::new(self)
    }

    pub fn with_source(mut self, filename: impl ToString, code: impl ToString) -> Box<Self> {
        self.sources
            .get_or_insert_with(Vec::new)
            .push((filename.to_string(), code.to_string()));
        Box::new(self)
    }

    pub fn report(&self) {
        let sources = self
            .sources
            .as_ref()
            .expect("Cannot report error without code base");
        let mut files = SimpleFiles::new();

        // Добавляем все исходники в SimpleFiles, запоминаем file_id для каждого
        let mut file_ids = Vec::new();
        for (name, code) in sources {
            let id = files.add(name.clone(), code.clone());
            file_ids.push(id);
        }

        // Предположим, что главный источник — последний
        let main_file_id = *file_ids.last().unwrap();

        // Собираем метки для Diagnostic
        let labels: Vec<Label<usize>> = self
            .labels
            .iter()
            .rev()
            .map(|label| {
                let file_index = file_ids
                    .iter()
                    .position(|&id| {
                        // Проверяем соответствие по имени
                        files
                            .name(id)
                            .map(|s| s == label.source_id)
                            .unwrap_or(false)
                    })
                    .unwrap_or(main_file_id);

                Label::new(label.label_style, file_ids[file_index], label.span.clone())
                    .with_message(label.label.clone())
            })
            .collect();

        // Формируем Diagnostic
        let mut diagnostic = Diagnostic::new(self.kind.severity).with_labels(labels);

        if let Some(code) = self.kind.code {
            diagnostic = diagnostic.with_code(code);
        }

        if let Some(msg) = &self.message {
            diagnostic = diagnostic.with_message(msg.clone());
        }

        for addition in &self.additions {
            match addition {
                RuneWayAddition::Help(help) => {
                    diagnostic =
                        diagnostic.with_note(format!("{} {}", "help:".bright_cyan(), help.clone()))
                }
                RuneWayAddition::Note(note) => {
                    diagnostic =
                        diagnostic.with_note(format!("{} {}", "note:".cyan(), note.clone()))
                }
            }
        }

        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = term::Config::default();

        println!();

        term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
    }
}

impl fmt::Display for RuneWayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: {}",
            self.kind.severity,
            self.message.as_ref().unwrap_or(&"".to_string())
        )
    }
}

#[derive(Debug, Clone)]
pub struct RuneWayErrorKind {
    pub severity: Severity,
    pub code: Option<&'static str>,
}

impl RuneWayErrorKind {
    #[inline]
    pub fn new(severity: Severity, code: Option<&'static str>) -> Self {
        Self { severity, code }
    }

    #[inline]
    pub fn error() -> Self {
        Self::new(Severity::Error, None)
    }

    #[inline(always)]
    pub fn error_with_code(code: &'static str) -> Self {
        Self::new(Severity::Error, Some(code))
    }

    #[inline(always)]
    pub fn warning() -> Self {
        Self::new(Severity::Warning, None)
    }

    #[inline(always)]
    pub fn warning_with_code(code: &'static str) -> Self {
        Self::new(Severity::Warning, Some(code))
    }

    // Basics
    #[inline(always)]
    pub fn syntax_error() -> Self {
        Self::error_with_code("SyntaxError")
    }

    #[inline(always)]
    pub fn runtime_error() -> Self {
        Self::error_with_code("RuntimeError")
    }

    #[inline(always)]
    pub fn type_error() -> Self {
        Self::error_with_code("TypeError")
    }

    #[inline(always)]
    pub fn name_error() -> Self {
        Self::error_with_code("NameError")
    }
}

#[derive(Debug, Clone)]
struct RuneWayErrorLabel {
    pub span: Range<usize>,
    pub label: String,
    pub source_id: String,
    pub label_style: LabelStyle,
}

#[derive(Debug, Clone)]
enum RuneWayAddition {
    Note(String),
    Help(String),
}

pub type RWResult<T> = Result<T, Box<RuneWayError>>;
