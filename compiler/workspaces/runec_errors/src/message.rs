use std::collections::HashMap;
use fluent::FluentValue;

#[derive(Debug)]
pub struct DiagMessage<'diag> {
    pub message_id: &'static str,
    pub args: Option<HashMap<&'static str, FluentValue<'diag>>>,
}

impl<'diag> DiagMessage<'diag> {
    impl_message_new!('diag);
    impl_add_arg!('diag);
}
