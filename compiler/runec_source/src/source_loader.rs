use std::fs::File;
use std::path::PathBuf;
use memmap2::Mmap;
use crate::source_map::{Source, SourceLineStarts};

#[derive(Debug)]
pub enum FileLoaderError {
    IoError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
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
    // Errors: std::io::Error, std::str::Utf8Error
    fn load(&self, path: PathBuf) -> Result<Source, FileLoaderError> {
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let source = str::from_utf8(&mmap)?;
        Ok(Source::File {
            lines: SourceLineStarts::compute_from_source(source),
            path,
            mmap,
        })
    }
}
