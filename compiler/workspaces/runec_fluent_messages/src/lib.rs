use fluent::{FluentResource};
use fluent::bundle::FluentBundle;
use once_cell::sync::Lazy;
use unic_langid::langids;

macro_rules! include_resources {
    ($($file:expr),* $(,)?) => {
        [$(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../resources/fluent/", $file))),*]
    };
}

static FLUENT_BUNDLE: Lazy<FluentBundle<FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>> = Lazy::new(|| {
    let lang_ids = langids!["en-US"];
    let mut bundle = FluentBundle::new_concurrent(lang_ids);

    for resource_file_text in include_resources![
        "lexer_messages.ftl",
    ] {
        let res = FluentResource::try_new(resource_file_text.into())
            .expect("Failed to parse an FTL string.");
        bundle.add_resource(res)
            .expect("Failed to add FTL resources to the bundle.");
    }

    bundle
});
