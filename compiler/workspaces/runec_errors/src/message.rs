use std::collections::HashMap;
use fluent::FluentValue;
use crate::{impl_add_arg, impl_message_new};

#[derive(Debug)]
pub struct DiagMessage<'a> {
    pub message_id: &'static str,
    pub args: Option<HashMap<&'static str, FluentValue<'a>>>,
}

impl<'a> DiagMessage<'a> {
    impl_message_new!();
    impl_add_arg!();
}
