macro_rules! include_resources {
    ($($file:expr),* $(,)?) => {
        &[$(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../resources/", $file))),*]
    };
}

static FLUENT_MESSAGE_RESOURCES: &[&'static str] = include_resources![
    "lexer_messages.ftl",
];

