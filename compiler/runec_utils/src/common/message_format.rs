use regex::{Regex, Captures};
use once_cell::sync::Lazy;

static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{\{(.+?)}}|\{(.+?)}").unwrap()
});

pub fn message_format(message: &str, replaces: &[(&str, &str)]) -> String {
    RE.replace_all(message, |caps: &Captures| {
        if let Some(m) = caps.get(1) {
            return format!("{{{}}}", m.as_str().trim());
        }

        if let Some(m) = caps.get(2) {
            let key = m.as_str().trim();
            return replaces.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.to_string())
                .unwrap_or_else(|| format!("{{{}}}", key));
        }

        caps[0].to_string()
    }).into_owned()
}
