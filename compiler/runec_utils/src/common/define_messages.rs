#[macro_export]
macro_rules! define_messages {
    ($($name:ident => $msg:expr),* $(,)?) => {
        $(pub const $name: &'static str = $msg;)*
    };
}
