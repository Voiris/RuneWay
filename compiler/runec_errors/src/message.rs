#[derive(Debug)]
pub struct DiagMessage {
    pub message: String,
}

impl DiagMessage {
    impl_message_new!();
}
