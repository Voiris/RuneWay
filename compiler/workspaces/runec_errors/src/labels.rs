use std::collections::HashMap;
use std::fmt::Display;
use fluent::FluentValue;
use runec_source::span::Span;

#[derive(Debug)]
pub enum DiagLabelKind {
    Primary,
    Secondary,
}

impl DiagLabelKind {
    pub fn marker(&self) -> char {
        match self {
            DiagLabelKind::Primary => '-',
            DiagLabelKind::Secondary => '^',
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            DiagLabelKind::Primary => "\x1b[1;96m",
            DiagLabelKind::Secondary => "\x1b[1;93m",
        }
    }
}

#[derive(Debug)]
pub struct DiagLabel<'a> {
    pub message_id: Option<&'static str>,
    pub args: Option<HashMap<&'static str, FluentValue<'a>>>,
    pub kind: DiagLabelKind,
    pub span: Span
}

#[derive(Debug)]
pub struct DiagNote<'a> {
    pub message_id: &'static str,
    pub args: Option<HashMap<&'static str, FluentValue<'a>>>,
}

#[derive(Debug)]
pub struct DiagHelp<'a> {
    pub message_id: &'static str,
    pub args: Option<HashMap<&'static str, FluentValue<'a>>>,
}
