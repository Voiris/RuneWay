use std::path::PathBuf;
use runec_source::source_map::{FileName, SourceFile, SourceId, SourceMap};

pub(crate) mod lexer;
pub mod parser;

#[cfg(test)]
fn generate_source(source: &str) -> (SourceMap, SourceId) {
    let mut source_map = SourceMap::new();
    let source_id = source_map.add_file(
        SourceFile::new(FileName::Real(PathBuf::from("/home/user/main.rnw")), source.to_string()),
    );
    (source_map, source_id)
}

