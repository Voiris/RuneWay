use std::path::PathBuf;
use runec_source::source_loader::FileLoaderError;
use runec_source::source_map::{Source, SourceLineStarts};

pub struct MockSourceFileLoader<'src> { pub source: &'src str }
impl<'src> MockSourceFileLoader<'src> {
    pub fn load(&self, path: PathBuf) -> Result<Source, FileLoaderError> {
        let mut mmap = memmap2::MmapOptions::new().len(self.source.len()).map_anon()?;

        mmap[..].copy_from_slice(self.source.as_bytes());

        let ro_mmap = mmap.make_read_only()?;

        Ok(Source::File {
            path,
            mmap: ro_mmap,
            lines: SourceLineStarts::compute_from_source(self.source),
        })
    }
}
