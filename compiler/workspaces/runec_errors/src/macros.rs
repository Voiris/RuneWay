#[macro_export]
macro_rules! impl_add_arg {
    () => {
        pub fn add_arg(mut self, key: &'static str, arg: FluentValue<'a>) -> Self {
            self.args.get_or_insert_with(HashMap::new).insert(key, arg);
            self
        }
    }
}

#[macro_export]
macro_rules! impl_message_new {
    () => {
        pub fn new(message_id: &'static str, args: Option<HashMap<&'static str, FluentValue<'a>>>) -> Self {
            Self { message_id, args }
        }

        pub fn new_simple(message_id: &'static str) -> Self {
            Self::new(message_id, None)
        }
    }
}
