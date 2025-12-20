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

impl<'a> DiagLabel<'a> {
    pub fn new(message_id: Option<&'static str>, args: Option<HashMap<&'static str, FluentValue<'a>>>, kind: DiagLabelKind, span: Span) -> Self {
        Self {
            message_id,
            args,
            kind,
            span,
        }
    }

    pub fn primary(message_id: Option<&'static str>, args: HashMap<&'static str, FluentValue<'a>>, span: Span) -> Self {
        Self::new(message_id, Some(args), DiagLabelKind::Primary, span)
    }

    pub fn simple_primary(message_id: &'static str, span: Span) -> Self {
        Self::new(Some(message_id), None, DiagLabelKind::Primary, span)
    }

    pub fn silent_primary(span: Span) -> Self {
        Self::new(None, None, DiagLabelKind::Primary, span)
    }

    pub fn secondary(message_id: Option<&'static str>, args: HashMap<&'static str, FluentValue<'a>>, span: Span) -> Self {
        Self::new(message_id, Some(args), DiagLabelKind::Secondary, span)
    }

    pub fn simple_secondary(message_id: &'static str, span: Span) -> Self {
        Self::new(Some(message_id), None, DiagLabelKind::Secondary, span)
    }

    pub fn silent_secondary(span: Span) -> Self {
        Self::new(None, None, DiagLabelKind::Secondary, span)
    }

    impl_add_arg!();
}

#[derive(Debug)]
pub struct DiagNote<'a> {
    pub message_id: &'static str,
    pub args: Option<HashMap<&'static str, FluentValue<'a>>>,
}

impl<'a> DiagNote<'a> {
    impl_message_new!();
    impl_add_arg!();
}

#[derive(Debug)]
pub struct DiagHelp<'a> {
    pub message_id: &'static str,
    pub args: Option<HashMap<&'static str, FluentValue<'a>>>,
}

impl<'a> DiagHelp<'a> {
    impl_message_new!();
    impl_add_arg!();
}
