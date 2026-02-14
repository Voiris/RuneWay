use std::path::PathBuf;
use runec_source::source_map::{Source, SourceId, SourceMap};

pub(crate) mod lexer;
pub mod parser;

#[cfg(test)]
fn generate_source(source: &str) -> (SourceMap, SourceId) {
    let mut source_map = SourceMap::new();

    let mock = runec_test_utils::MockSourceFileLoader { source };

    let source_id = source_map.add_file(
        mock.load(PathBuf::from("/home/user/main.rnw")).unwrap()
    );
    (source_map, source_id)
}
