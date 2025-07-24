mod buffered;
mod http;
mod json;

pub fn prelude() {
    buffered::register();
    http::register();
    json::register();
}
