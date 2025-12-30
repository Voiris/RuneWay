use std::collections::HashMap;
use fluent::FluentValue;
use crate::{impl_add_arg, impl_message_new};

#[derive(Debug)]
pub struct DiagMessage<'diag> {
    pub message_id: &'static str,
    pub args: Option<HashMap<&'static str, FluentValue<'diag>>>,
}

impl<'diag> DiagMessage<'diag> {
    impl_message_new!('diag);
    impl_add_arg!('diag);
}
