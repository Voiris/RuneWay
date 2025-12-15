use std::path::PathBuf;
use std::sync::Arc;
use crate::byte_pos::BytePos;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceId(u16);

impl SourceId {
    pub const fn from_usize(n: usize) -> SourceId {
        SourceId(n as u16)
    }

    pub const fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineIndex(u16);

impl LineIndex {
    pub const fn from_usize(n: usize) -> LineIndex {
        LineIndex(n as u16)
    }

    pub const fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

/// Stores the start position of each line in the source file
pub struct SourceLineStarts(Box<[BytePos]>);

impl SourceLineStarts {
    pub fn new(starts: Vec<BytePos>) -> SourceLineStarts {
        SourceLineStarts(starts.into_boxed_slice())
    }

    /// Computes line start positions from a source string
    ///
    /// Each line start is represented by the byte index of its first character.
    /// The first line always starts at byte position 0.
    pub fn compute_from_source(src: &str) -> SourceLineStarts {
        let mut line_starts = vec![BytePos::from_usize(0)];

        // Find all newline characters and record the start of the next line
        for pos in memchr::memchr_iter(b'\n', src.as_bytes()) {
            line_starts.push(BytePos::from_usize(pos + 1));
        }

        SourceLineStarts(line_starts.into_boxed_slice())
    }

    /// Returns a slice of start positions of all lines
    pub fn get(&self) -> &[BytePos] {
        &self.0
    }

    /// Finds the line corresponding to a given byte position.
    ///
    /// Returns a tuple `(LineIndex, BytePos)`:
    /// - `LineIndex` is the 0-based index of the line
    /// - `BytePos` is the start position of that line
    ///
    /// If `byte_pos` exactly matches a line start, that line is returned.
    /// If `byte_pos` falls between lines, the previous line is returned.
    pub fn line_search(&self, byte_pos: BytePos) -> (LineIndex, BytePos) {
        let slice = self.get();
        match slice.binary_search(&byte_pos) {
            Ok(idx) => (LineIndex::from_usize(idx), slice[idx]),
            Err(idx) => (LineIndex::from_usize(idx - 1), slice[idx - 1]),
        }
    }
}

pub enum FileName {
    Real(PathBuf),
    // Maybe other... Maybe later...
}

pub struct SourceFile {
    pub file_name: FileName,
    pub src: Arc<String>,
    pub lines: SourceLineStarts,
}

impl SourceFile {
    pub const MAX_FILE_SIZE: usize = BytePos::MAX;

    pub fn new(file_name: FileName, src: String) -> SourceFile {
        SourceFile {
            file_name,
            lines: SourceLineStarts::compute_from_source(&src),
            src: Arc::new(src),
        }
    }
}

pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> SourceMap {
        SourceMap { files: Vec::new() }
    }

    pub fn add_file(&mut self, source_file: SourceFile) -> SourceId {
        let new_id = SourceId::from_usize(self.files.len());
        self.files.push(source_file);
        new_id
    }

    pub fn get_file(&self, id: SourceId) -> Option<&SourceFile> {
        self.files.get(id.to_usize())
    }
}

#[cfg(test)]
mod tests {
    use crate::byte_pos::BytePos;
    use crate::source_map::{LineIndex, SourceLineStarts};

    #[test]
    fn test_line_search() {
        let source_line_starts = SourceLineStarts::new(
            vec![
                BytePos::from_usize(0),
                BytePos::from_usize(10),
                BytePos::from_usize(20),
                BytePos::from_usize(30)
            ]
        );

        // Equal search
        assert_eq!(source_line_starts.line_search(BytePos::from_usize(0)), (LineIndex::from_usize(0), BytePos::from_usize(0)));
        assert_eq!(source_line_starts.line_search(BytePos::from_usize(10)), (LineIndex::from_usize(1), BytePos::from_usize(10)));
        assert_eq!(source_line_starts.line_search(BytePos::from_usize(20)), (LineIndex::from_usize(2), BytePos::from_usize(20)));
        assert_eq!(source_line_starts.line_search(BytePos::from_usize(30)), (LineIndex::from_usize(3), BytePos::from_usize(30)));

        // Greater search
        // Check positions between lines
        for i in 1..10 {
            assert_eq!(source_line_starts.line_search(BytePos::from_usize(i)), (LineIndex::from_usize(0), BytePos::from_usize(0)));
            assert_eq!(source_line_starts.line_search(BytePos::from_usize(10 + i)), (LineIndex::from_usize(1), BytePos::from_usize(10)));
            assert_eq!(source_line_starts.line_search(BytePos::from_usize(20 + i)), (LineIndex::from_usize(2), BytePos::from_usize(20)));
            assert_eq!(source_line_starts.line_search(BytePos::from_usize(30 + i)), (LineIndex::from_usize(3), BytePos::from_usize(30)));
        }
    }
}
