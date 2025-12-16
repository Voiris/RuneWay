use std::collections::HashMap;
use fluent::FluentValue;

#[derive(Debug)]
pub struct DiagMessage<'a> {
    pub message_id: &'static str,
    pub args: Option<HashMap<&'static str, FluentValue<'a>>>,
}
