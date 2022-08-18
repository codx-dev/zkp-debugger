mod context;

use std::borrow::Borrow;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, Write};
use std::path::Path;

pub use context::EncoderContext;
use msgpacker::Message;

use crate::{Config, EncodableConstraint, EncodableElement, EncodableWitness, Preamble};

/// An encoder for CDF format
#[derive(Debug)]
pub struct Encoder<WI, CI, T> {
    context: EncoderContext,
    witnesses: WI,
    constraints: CI,
    target: T,
}

impl<WI, CI, T> Encoder<WI, CI, T>
where
    WI: ExactSizeIterator,
    CI: ExactSizeIterator,
{
    pub(crate) fn with_preamble(
        preamble: Preamble,
        witnesses: WI,
        constraints: CI,
        target: T,
    ) -> Self {
        let context = EncoderContext::from_preamble(preamble);

        Self {
            context,
            witnesses,
            constraints,
            target,
        }
    }

    /// Create a new encoder
    pub(crate) fn new(config: Config, witnesses: WI, constraints: CI, target: T) -> Self {
        let preamble = Preamble::new(witnesses.len(), constraints.len(), config);

        Self::with_preamble(preamble, witnesses, constraints, target)
    }

    /// Return the underlying encoder
    pub fn into_inner(self) -> T {
        self.target
    }
}

impl<WI, CI> Encoder<WI, CI, File>
where
    WI: ExactSizeIterator,
    CI: ExactSizeIterator,
{
    /// Initialize the encoder, filling a file with required bytes.
    ///
    /// Check [`File::set_len`]
    pub fn init_file<P>(config: Config, witnesses: WI, constraints: CI, path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = OpenOptions::new().write(true).create(true).open(path)?;

        let slf = Self::new(config, witnesses, constraints, file);
        let len = slf.context.preamble().source_cache_offset();

        slf.target.set_len(len as u64)?;

        Ok(slf)
    }
}

impl<WI, CI, B> Encoder<WI, CI, io::BufWriter<B>>
where
    WI: ExactSizeIterator,
    CI: ExactSizeIterator,
    B: io::Write + io::Seek,
{
    /// Initialize the encoder, filling the buffer with required bytes.
    pub fn init_buffer(
        config: Config,
        witnesses: WI,
        constraints: CI,
        buffer: B,
    ) -> io::Result<Self> {
        let buffer = io::BufWriter::new(buffer);
        let mut slf = Self::new(config, witnesses, constraints, buffer);
        let len = slf.context.preamble().source_cache_offset();

        let n = slf
            .target
            .rewind()
            .and_then(|_| slf.target.write(&vec![0u8; len]))?;

        if n != len {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "the target wrote {} bytes instead of the expected {}!",
                    n, len
                ),
            ));
        }

        Ok(slf)
    }
}

impl<WI, CI> Encoder<WI, CI, io::Cursor<Vec<u8>>>
where
    WI: ExactSizeIterator,
    CI: ExactSizeIterator,
{
    /// Initialize the encoder, filling the buffer with required bytes.
    pub fn init_cursor(config: Config, witnesses: WI, constraints: CI) -> Self {
        let preamble = Preamble::new(witnesses.len(), constraints.len(), config);
        let len = preamble.source_cache_offset();
        let bytes = vec![0u8; len];
        let cursor = io::Cursor::new(bytes);

        Self::with_preamble(preamble, witnesses, constraints, cursor)
    }
}

impl<W, WI, C, CI, T> Encoder<WI, CI, T>
where
    W: Borrow<EncodableWitness>,
    WI: Iterator<Item = W> + ExactSizeIterator,
    C: Borrow<EncodableConstraint>,
    CI: Iterator<Item = C> + ExactSizeIterator,
    T: io::Write + io::Seek,
{
    /// Write all witnesses and constraints into the target
    pub fn write_all(&mut self) -> io::Result<usize> {
        let Self {
            context,
            witnesses,
            constraints,
            target,
        } = self;

        let preamble = context.preamble().clone();
        let n = preamble.try_to_writer(target.by_ref(), context)?;

        let n = witnesses.try_fold(n, |n, w| {
            w.borrow()
                .try_to_writer(target.by_ref(), context)
                .map(|x| n + x)
        })?;

        let n = constraints.try_fold(n, |n, c| {
            c.borrow()
                .try_to_writer(target.by_ref(), context)
                .map(|x| n + x)
        })?;

        let mut source_cache = self.context.iter().collect::<Vec<_>>();

        source_cache.as_mut_slice().sort_by_key(|(_path, idx)| *idx);

        let source_cache_file_names = source_cache
            .into_iter()
            .map(|(path, _idx)| path.canonicalize())
            .collect::<io::Result<Vec<_>>>()?;

        let source_cache_contents = source_cache_file_names
            .iter()
            .map(fs::read_to_string)
            .collect::<io::Result<Vec<_>>>()?
            .into_iter()
            .map(Message::String)
            .collect::<Vec<_>>();

        let source_cache_file_names = source_cache_file_names
            .into_iter()
            .map(|path| format!("{}", path.display()))
            .map(Message::String)
            .collect::<Vec<_>>();

        let n = n + Message::Array(source_cache_file_names).pack(target)?;
        let n = n + Message::Array(source_cache_contents).pack(target)?;

        Ok(n)
    }
}
