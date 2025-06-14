pub trait RuneWayErrorKind {
    fn description(&self) -> &str;
}

#[derive(Debug)]
pub struct RuneWayError<K: RuneWayErrorKind> {
    pub kind: K,
    pub message: String,
    pub line: usize,
}

impl<K: RuneWayErrorKind> RuneWayError<K> {
    pub fn new(kind: K, message: String, line: usize) -> Self {
        Self { kind, message, line }
    }
}
