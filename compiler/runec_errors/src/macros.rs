macro_rules! impl_message_new {
    () => {
        pub fn new(message: &'static str, args: &[(&str, &str)]) -> Self {
            Self { message: runec_utils::common::message_format::message_format(message, args) }
        }
    }
}
