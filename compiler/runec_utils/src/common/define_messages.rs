#[macro_export]
macro_rules! define_message {
    ($($name:ident => $msg:expr),* $(,)?) => {
        $(pub const $name: &'static str = $msg;)*
    };
}
