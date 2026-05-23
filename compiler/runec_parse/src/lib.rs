extern crate core;

#[cfg(test)]
use runec_source::source_map::{SourceId, SourceMap};
#[cfg(test)]
use std::path::PathBuf;

pub mod lexer;
mod messages;
pub mod parser;

pub use lexer::lexer_struct::Lexer;
pub use lexer::token::{Radix, SpannedToken, Token};
pub use parser::{ParseResult, Parser};

#[cfg(test)]
fn generate_source(source: &str) -> (SourceMap, SourceId) {
    let mut source_map = SourceMap::new();

    let mock = runec_test_utils::MockSourceFileLoader { source };

    let source_id = source_map.add_file(mock.load(PathBuf::from("/home/user/main.rnw")).unwrap());
    (source_map, source_id)
}
