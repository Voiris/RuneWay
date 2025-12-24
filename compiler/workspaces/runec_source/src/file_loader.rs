use std::fs::File;
use std::io;
use std::path::Path;
use std::io::{Read, Result as IoResult};
use crate::source_map::SourceFile;

pub trait FileLoader {
    /// Check the file existence
    fn file_exists(&self, path: &Path) -> bool;

    /// Read the content of a UTF-8 file
    fn read_file(&self, path: &Path) -> IoResult<String>;
}


/// std::fs based file loader
pub struct RealFileLoader;

impl FileLoader for RealFileLoader {
    fn file_exists(&self, path: &Path) -> bool {
        // Checks if the path exists and is a file
        path.is_file()
    }

    fn read_file(&self, path: &Path) -> IoResult<String> {
        let mut file = File::open(path)?;

        if file.metadata()?.len() > SourceFile::MAX_FILE_SIZE as u64 {
            return Err(io::Error::other(
                format!("File {} is too large. Supported: {} bytes", path.display(), SourceFile::MAX_FILE_SIZE),
            ));
        };

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        Ok(buffer)
    }
}

