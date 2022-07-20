use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use crate::{AtomicConfig, Config, Element, FixedText, Preamble};

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
        2 * u64::len(&AtomicConfig) + Self::PATH_LEN as usize
    }

    fn to_buffer(&self, config: &Self::Config, buf: &mut [u8]) {
        let buf = self.line.encode(&AtomicConfig, buf);
        let buf = self.col.encode(&AtomicConfig, buf);
        let _ = self.path.encode(config, buf);
    }

    fn try_from_buffer_in_place(&mut self, config: &Self::Config, buf: &[u8]) -> io::Result<()> {
        let buf = self.line.try_decode_in_place(&AtomicConfig, buf)?;
        let buf = self.col.try_decode_in_place(&AtomicConfig, buf)?;
        let _ = self.path.try_decode_in_place(config, buf)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.line.validate(preamble)?;
        self.col.validate(preamble)?;
        self.path.validate(preamble)?;

        Ok(())
    }
}
