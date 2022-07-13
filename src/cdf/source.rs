use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use bat::line_range::LineRanges;
use bat::PrettyPrinter;

use super::{Element, FixedText, Preamble};

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

    /// Return the canonical path represented in the source file. Read more: [`Path::canonicalize`]
    pub fn canonical_path(&self) -> io::Result<PathBuf> {
        Path::new(self.path.as_str()).canonicalize()
    }

    /// Open the source file as read-only
    pub fn open(&self) -> io::Result<File> {
        self.canonical_path()
            .and_then(|path| OpenOptions::new().read(true).open(path))
    }

    /// Render the file into the terminal
    pub fn render(&self, margin: usize) -> io::Result<()> {
        let path = self.canonical_path()?;

        let line = self.line as usize;
        let range = LineRanges::from(vec![bat::line_range::LineRange::new(
            line.saturating_sub(margin),
            line.saturating_add(margin),
        )]);

        PrettyPrinter::new()
            .input_file(path)
            .language("rust")
            .header(true)
            .grid(true)
            .line_numbers(true)
            .line_ranges(range)
            .highlight(line)
            .print()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }
}

impl Element for Source {
    const LEN: usize = 2 * u64::LEN + Self::PATH_LEN as usize;

    fn zeroed() -> Self {
        Self::default()
    }

    fn to_buffer(&self, buf: &mut [u8]) {
        let buf = self.line.encode(buf);
        let buf = self.col.encode(buf);
        let _ = self.path.encode(buf);
    }

    fn try_from_buffer_in_place(&mut self, buf: &[u8]) -> io::Result<()> {
        let buf = self.line.try_decode_in_place(buf)?;
        let buf = self.col.try_decode_in_place(buf)?;
        let _ = self.path.try_decode_in_place(buf)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.line.validate(preamble)?;
        self.col.validate(preamble)?;
        self.path.validate(preamble)?;

        Ok(())
    }
}

impl io::Write for Source {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.try_write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.path.flush()?;

        Ok(())
    }
}

impl io::Read for Source {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.try_read(buf)
    }
}
