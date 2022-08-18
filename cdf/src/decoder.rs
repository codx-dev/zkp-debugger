mod context;

use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

pub use context::DecoderContext;
use msgpacker::Message;

use crate::{Constraint, DecodableElement, Preamble, Witness};

/// A circuit description file
///
/// Since circuit descriptions are often large, it will perform lazy disk I/O, loading only the
/// required data to satisfy the user operation.
#[derive(Debug, Clone)]
pub struct CircuitDescription<S> {
    preamble: Preamble,
    source_names: Vec<String>,
    source_contents: Vec<String>,
    source: S,
}

impl<S> CircuitDescription<S> {
    pub(crate) fn context(&mut self) -> (DecoderContext, &mut S) {
        let Self {
            preamble,
            source_names,
            source_contents,
            source,
        } = self;

        let ctx = DecoderContext::new(&preamble.config, source_names, source_contents);

        (ctx, source)
    }

    /// Decoded preamble
    pub const fn preamble(&self) -> &Preamble {
        &self.preamble
    }
}

impl CircuitDescription<File> {
    /// Open a CDF file as read-only.
    pub fn open<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        OpenOptions::new()
            .read(true)
            .open(path)
            .and_then(Self::from_reader)
    }
}

impl<S> CircuitDescription<S>
where
    S: io::Read + io::Seek,
{
    /// Create a new circuit description instance.
    pub fn from_reader(mut source: S) -> io::Result<Self> {
        // reset the cursor
        source.seek(io::SeekFrom::Start(0))?;

        // load the preamble with the base config
        let preamble = Preamble::try_from_reader(&DecoderContext::BASE, source.by_ref())?;

        let ofs = preamble.source_cache_offset();
        let ofs = io::SeekFrom::Start(ofs as u64);
        source.seek(ofs)?;

        let source_names = Message::unpack(source.by_ref())?;
        let source_contents = Message::unpack(source.by_ref())?;

        let (source_names, source_contents) = match (source_names, source_contents) {
            (Message::Array(n), Message::Array(c)) => (n, c),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "the source cache isn't a valid array",
                ))
            }
        };

        let source_names = source_names
            .into_iter()
            .map(|m| match m {
                Message::String(s) => Ok(s),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "the source names isn't composed of strings",
                )),
            })
            .collect::<io::Result<Vec<_>>>()?;

        let source_contents = source_contents
            .into_iter()
            .map(|m| match m {
                Message::String(s) => Ok(s),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "the source contents isn't composed of strings",
                )),
            })
            .collect::<io::Result<Vec<_>>>()?;

        Ok(Self {
            preamble,
            source_names,
            source_contents,
            source,
        })
    }

    /// Attempt to read an indexed constraint from the source
    pub fn fetch_constraint(&mut self, idx: usize) -> io::Result<Constraint> {
        self.preamble
            .constraint_offset(idx)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "attempt to fetch invalid constraint")
            })
            .map(|ofs| io::SeekFrom::Start(ofs as u64))
            .and_then(|ofs| self.source.seek(ofs))?;

        let (ctx, source) = self.context();

        Constraint::try_from_reader(&ctx, source)
    }

    /// Attempt to read an indexed witness from the source
    pub fn fetch_witness(&mut self, idx: usize) -> io::Result<Witness> {
        self.preamble
            .witness_offset(idx)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "attempt to fetch invalid witness"))
            .map(|ofs| io::SeekFrom::Start(ofs as u64))
            .and_then(|ofs| self.source.seek(ofs))?;

        let (ctx, source) = self.context();

        Witness::try_from_reader(&ctx, source)
    }
}
