use std::fmt::{self, Display};
use std::fs::File;
use std::path::PathBuf;

use memmap2::Mmap;

use crate::source_map::{Source, SourceLineStarts};

#[derive(Debug)]
pub enum FileLoaderError {
    IoError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
}

impl Display for FileLoaderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileLoaderError::IoError(error) => {
                write!(formatter, "failed to read source file: {error}")
            }
            FileLoaderError::Utf8Error(error) => {
                write!(formatter, "source file is not valid UTF-8: {error}")
            }
        }
    }
}

impl std::error::Error for FileLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FileLoaderError::IoError(error) => Some(error),
            FileLoaderError::Utf8Error(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for FileLoaderError {
    fn from(error: std::io::Error) -> Self {
        FileLoaderError::IoError(error)
    }
}

impl From<std::str::Utf8Error> for FileLoaderError {
    fn from(error: std::str::Utf8Error) -> Self {
        FileLoaderError::Utf8Error(error)
    }
}

pub struct SourceFileLoader;
impl SourceFileLoader {
    pub fn load(&self, path: PathBuf) -> Result<Source, FileLoaderError> {
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let source = str::from_utf8(&mmap)?;
        Ok(Source::File { lines: SourceLineStarts::compute_from_source(source), path, mmap })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use super::FileLoaderError;

    #[test]
    fn file_loader_error_exposes_context_and_source() {
        let error = FileLoaderError::from(io::Error::new(io::ErrorKind::NotFound, "missing"));

        assert_eq!(error.to_string(), "failed to read source file: missing");
        assert!(error.source().is_some());
    }
}
