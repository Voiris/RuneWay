mod buffered;
mod dynbox;
mod http;
mod itertools;
mod json;
mod random;

pub fn prelude() {
    buffered::register();
    http::register();
    json::register();
    random::register();
    itertools::register();
    dynbox::register();
}
