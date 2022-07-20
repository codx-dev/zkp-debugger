use core::borrow::Borrow;
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, Write};
use std::path::Path;

use crate::{Config, Constraint, Preamble, Witness, Element, AtomicConfig};

/// An encoder for CDF format
#[derive(Debug)]
pub struct Encoder<WI, CI, T> {
    preamble: Preamble,
    witnesses: WI,
    constraints: CI,
    target: T,
}

impl<WI, CI, T> Encoder<WI, CI, T> {
    /// Create a new encoder
    pub(crate) const fn new(preamble: Preamble, witnesses: WI, constraints: CI, target: T) -> Self {
        Self {
            preamble,
            witnesses,
            constraints,
            target,
        }
    }

    /// Return the inner target
    pub fn into_inner(self) -> T {
        self.target
    }
}

impl<W, WI, C, CI, T> Encoder<WI, CI, T>
where
    W: Borrow<Witness>,
    WI: Iterator<Item = W> + ExactSizeIterator,
    C: Borrow<Constraint>,
    CI: Iterator<Item = C> + ExactSizeIterator,
{
    /// Intialize a preamble from a config file
    pub fn init_preamble(config: Config, witnesses: &WI, constraints: &CI) -> Preamble {
        *Preamble::new()
            .with_config(config)
            .with_witnesses(witnesses.len())
            .with_constraints(constraints.len())
    }
}

impl<W, WI, C, CI> Encoder<WI, CI, File>
where
    W: Borrow<Witness>,
    WI: Iterator<Item = W> + ExactSizeIterator,
    C: Borrow<Constraint>,
    CI: Iterator<Item = C> + ExactSizeIterator,
{
    /// Initialize the encoder, filling a file with required bytes.
    ///
    /// Check [`File::set_len`]
    pub fn init_file<P>(config: Config, witnesses: WI, constraints: CI, path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let preamble = Self::init_preamble(config, &witnesses, &constraints);
        let len = preamble.total_len();

        let file = OpenOptions::new().write(true).create(true).open(path)?;

        file.set_len(len as u64)?;

        Ok(Self::new(preamble, witnesses, constraints, file))
    }
}

impl<W, WI, C, CI, B> Encoder<WI, CI, io::BufWriter<B>>
where
    W: Borrow<Witness>,
    WI: Iterator<Item = W> + ExactSizeIterator,
    C: Borrow<Constraint>,
    CI: Iterator<Item = C> + ExactSizeIterator,
    B: io::Write + io::Seek,
{
    /// Initialize the encoder, filling the buffer with required bytes.
    pub fn init_buffer(
        config: Config,
        witnesses: WI,
        constraints: CI,
        buffer: B,
    ) -> io::Result<Self> {
        let preamble = Self::init_preamble(config, &witnesses, &constraints);
        let len = preamble.total_len();

        let mut buffer = io::BufWriter::new(buffer);

        let n = buffer
            .rewind()
            .and_then(|_| buffer.write(&vec![0u8; len]))?;

        if n != len {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "the target wrote {} bytes instead of the expected {}!",
                    n, len
                ),
            ));
        }

        Ok(Self::new(preamble, witnesses, constraints, buffer))
    }
}

impl<W, WI, C, CI> Encoder<WI, CI, io::Cursor<Vec<u8>>>
where
    W: Borrow<Witness>,
    WI: Iterator<Item = W> + ExactSizeIterator,
    C: Borrow<Constraint>,
    CI: Iterator<Item = C> + ExactSizeIterator,
{
    /// Initialize the encoder, filling the cursor with required bytes.
    pub fn init_cursor(config: Config, witnesses: WI, constraints: CI) -> Self {
        let preamble = Self::init_preamble(config, &witnesses, &constraints);
        let len = preamble.total_len();

        let bytes = vec![0u8; len];
        let cursor = io::Cursor::new(bytes);

        Self::new(preamble, witnesses, constraints, cursor)
    }
}

impl<W, WI, C, CI, T> Encoder<WI, CI, T>
where
    W: Borrow<Witness>,
    WI: Iterator<Item = W> + ExactSizeIterator,
    C: Borrow<Constraint>,
    CI: Iterator<Item = C> + ExactSizeIterator,
    T: io::Write + io::Seek,
{
    /// Write all witnesses and constraints into the target
    pub fn write_all(&mut self) -> io::Result<usize>
    where
        W: Borrow<Witness>,
        WI: Iterator<Item = W>,
        C: Borrow<Constraint>,
        CI: Iterator<Item = C>,
    {
        let preamble = &self.preamble;
        let config = &self.preamble.config;
        let witnesses = &mut self.witnesses;
        let constraints = &mut self.constraints;
        let target = &mut self.target;

        let n = preamble.try_to_writer(target.by_ref(), &AtomicConfig)?;

        let n = witnesses.try_fold(n, |n, w| {
            let w = w.borrow();
            let id = w.id() as usize;

            preamble
                .witness_offset(id)
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to calculate offset"))
                .map(|ofs| io::SeekFrom::Start(ofs as u64))
                .and_then(|ofs| target.seek(ofs))
                .and_then(|_| w.try_to_writer(target.by_ref(), config))
                .map(|x| n + x)
        })?;

        let n = constraints.try_fold(n, |n, c| {
            let c = c.borrow();
            let id = c.id() as usize;

            preamble
                .constraint_offset(id)
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to calculate offset"))
                .map(|ofs| io::SeekFrom::Start(ofs as u64))
                .and_then(|ofs| target.seek(ofs))
                .and_then(|_| c.try_to_writer(target.by_ref(), config))
                .map(|x| n + x)
        })?;

        Ok(n)
    }
}
