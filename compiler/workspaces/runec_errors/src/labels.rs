use std::collections::HashMap;
use fluent::FluentValue;
use runec_source::span::Span;
use crate::{impl_add_arg, impl_message_new};

#[derive(Debug)]
pub enum DiagLabelKind {
    Primary,
    Secondary,
}

impl DiagLabelKind {
    pub fn marker(&self) -> char {
        match self {
            DiagLabelKind::Primary => '^',
            DiagLabelKind::Secondary => '-',
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
pub struct DiagLabel<'diag> {
    pub message_id: Option<&'static str>,
    pub args: Option<HashMap<&'static str, FluentValue<'diag>>>,
    pub kind: DiagLabelKind,
    pub span: Span
}

impl<'diag> DiagLabel<'diag> {
    pub fn new(message_id: Option<&'static str>, args: Option<HashMap<&'static str, FluentValue<'diag>>>, kind: DiagLabelKind, span: Span) -> Self {
        Self {
            message_id,
            args,
            kind,
            span,
        }
    }

    pub fn primary(message_id: Option<&'static str>, args: HashMap<&'static str, FluentValue<'diag>>, span: Span) -> Self {
        Self::new(message_id, Some(args), DiagLabelKind::Primary, span)
    }

    pub fn simple_primary(message_id: &'static str, span: Span) -> Self {
        Self::new(Some(message_id), None, DiagLabelKind::Primary, span)
    }

    pub fn silent_primary(span: Span) -> Self {
        Self::new(None, None, DiagLabelKind::Primary, span)
    }

    pub fn secondary(message_id: Option<&'static str>, args: HashMap<&'static str, FluentValue<'diag>>, span: Span) -> Self {
        Self::new(message_id, Some(args), DiagLabelKind::Secondary, span)
    }

    pub fn simple_secondary(message_id: &'static str, span: Span) -> Self {
        Self::new(Some(message_id), None, DiagLabelKind::Secondary, span)
    }

    pub fn silent_secondary(span: Span) -> Self {
        Self::new(None, None, DiagLabelKind::Secondary, span)
    }

    impl_add_arg!('diag);
}

#[derive(Debug)]
pub struct DiagNote<'diag> {
    pub message_id: &'static str,
    pub args: Option<HashMap<&'static str, FluentValue<'diag>>>,
}

impl<'diag> DiagNote<'diag> {
    impl_message_new!('diag);
    impl_add_arg!('diag);
}

#[derive(Debug)]
pub struct DiagHelp<'diag> {
    pub message_id: &'static str,
    pub args: Option<HashMap<&'static str, FluentValue<'diag>>>,
}

impl<'diag> DiagHelp<'diag> {
    impl_message_new!('diag);
    impl_add_arg!('diag);
}
