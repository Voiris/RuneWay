use std::collections::HashMap;
use fluent::FluentValue;
use runec_source::span::Span;

#[derive(Debug)]
pub struct DiagLabel<'a> {
    pub message_id: Option<&'static str>,
    pub args: Option<HashMap<&'static str, FluentValue<'a>>>,
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
