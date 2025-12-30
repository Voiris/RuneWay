use fluent::{FluentMessage, FluentResource};
use fluent::bundle::FluentBundle;
use once_cell::sync::Lazy;
use unic_langid::langids;

macro_rules! include_resources {
    ($($file:expr),* $(,)?) => {
        [$(($file, include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../resources/fluent/", $file)))),*]
    };
}

static FLUENT_BUNDLE: Lazy<FluentBundle<FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>> = Lazy::new(|| {
    let lang_ids = langids!["en-US"];
    let mut bundle = FluentBundle::new_concurrent(lang_ids);
    bundle.set_use_isolating(false);

    for (file_name, text) in include_resources![
        "lexer_messages.ftl",
        "test_messages.ftl",
    ] {
        let res = FluentResource::try_new(text.into()).unwrap_or_else(|_| panic!("Failed to parse {}", file_name));
        bundle.add_resource(res)
            .expect("Failed to add FTL resources to the bundle.");
    }

    bundle
});

pub fn get_fluent_bundle() -> &'static FluentBundle<FluentResource, intl_memoizer::concurrent::IntlLangMemoizer> {
    &FLUENT_BUNDLE
}

pub fn get_fluent_message(id: &'static str) -> FluentMessage<'static> {
    FLUENT_BUNDLE.get_message(id).unwrap_or_else(|| panic!("Fluent message `{}` not found", id))
}
