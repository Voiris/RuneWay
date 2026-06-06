use once_cell::sync::Lazy;
use regex::{Captures, Regex};

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{(.*?)}}|\{(.*?)}").unwrap());

pub fn message_format(message: &str, replaces: &[(&str, &str)]) -> String {
    RE.replace_all(message, |caps: &Captures| {
        if let Some(m) = caps.get(1) {
            return format!("{{{}}}", m.as_str().trim());
        }

        if let Some(m) = caps.get(2) {
            let key = m.as_str().trim();
            return replaces
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.to_string())
                .unwrap_or_else(|| format!("{{{}}}", key));
        }

        caps[0].to_string()
    })
    .into_owned()
}

#[cfg(test)]
mod tests {
    use super::message_format;

    #[test]
    fn replaces_named_placeholders() {
        assert_eq!(
            message_format(
                "expected {expected}, found {actual}",
                &[("expected", "integer"), ("actual", "string"),]
            ),
            "expected integer, found string"
        );
    }

    #[test]
    fn preserves_unknown_placeholders() {
        assert_eq!(message_format("unknown { name }", &[]), "unknown {name}");
    }

    #[test]
    fn unescapes_double_braces_including_empty_braces() {
        assert_eq!(
            message_format("literal {{name}} and {{}}", &[("name", "value")]),
            "literal {name} and {}"
        );
    }

    #[test]
    fn preserves_empty_placeholder() {
        assert_eq!(message_format("empty {}", &[]), "empty {}");
    }
}
