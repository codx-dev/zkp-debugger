use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use crate::{AtomicConfig, Config, Context, ContextUnit, Element, FixedText, Preamble};

/// Source file representation for debug mapping, including line and column of a file
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Source {
    line: u64,
    col: u64,
    path: FixedText<{ Self::PATH_LEN }>,
}

impl Source {
    /// Maximum allowed path length
    pub const PATH_LEN: u16 = 1024;

    /// Create a new source
    pub const fn new(line: u64, col: u64, path: FixedText<{ Self::PATH_LEN }>) -> Self {
        Self { line, col, path }
    }

    /// Source path
    pub const fn path(&self) -> &FixedText<{ Self::PATH_LEN }> {
        &self.path
    }

    /// Source line
    pub const fn line(&self) -> u64 {
        self.line
    }

    /// Return the canonical path represented in the source file. Read more: [`Path::canonicalize`]
    pub fn canonical_path(&self) -> io::Result<PathBuf> {
        Path::new(&*self.path).canonicalize()
    }

    /// Open the source file as read-only
    pub fn open(&self) -> io::Result<File> {
        self.canonical_path()
            .and_then(|path| OpenOptions::new().read(true).open(path))
    }
}

impl Element for Source {
    type Config = Config;

    fn zeroed() -> Self {
        Self::default()
    }

    fn len(_config: &Self::Config) -> usize {
        3 * u64::len(&AtomicConfig)
    }

    fn to_buffer(&self, _config: &Self::Config, context: &mut ContextUnit, buf: &mut [u8]) {
        let source_id = context.take_source_cache_id().unwrap_or_default();

        let buf = self.line.encode(&AtomicConfig, context, buf);
        let buf = self.col.encode(&AtomicConfig, context, buf);
        let _ = source_id.encode(&AtomicConfig, context, buf);
    }

    fn try_from_buffer_in_place<S>(
        &mut self,
        config: &Self::Config,
        context: &mut Context<S>,
        buf: &[u8],
    ) -> io::Result<()>
    where
        S: io::Read + io::Seek,
    {
        Self::validate_buffer_len(config, buf.len())?;

        let mut source_id = 0usize;

        let buf = self.line.try_decode_in_place(&AtomicConfig, context, buf)?;
        let buf = self.col.try_decode_in_place(&AtomicConfig, context, buf)?;
        let _ = source_id.try_decode_in_place(&AtomicConfig, context, buf)?;

        self.path = context.fetch_source_path(source_id)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.line.validate(preamble)?;
        self.col.validate(preamble)?;
        self.path.validate(preamble)?;

        Ok(())
    }
}

#[test]
fn open_canonical_path_works() {
    use std::fs;

    let dir = tempfile::tempdir().expect("failed to create temporary dir");
    let file = dir.path().join("open-canon-path-works.txt");

    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&file)
        .expect("failed to open new file");

    let path = FixedText::from(format!("{}", file.display()));

    let line = 20;
    let col = 5;

    let source = Source::new(line, col, path);

    let canon = file.canonicalize().expect("failed to canon original path");
    let canonical = source
        .canonical_path()
        .expect("failed to open canonical path");

    assert_eq!(canon, canonical);

    source.open().expect("failed to open source path");

    fs::remove_dir_all(dir).ok();
}
