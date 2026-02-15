extern crate core;

#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use runec_source::source_map::{SourceId, SourceMap};

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
