#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

//! The binary format for CDF is a dense linear encoding.
//!
//! A circuit is a compositions of items that are either [`Witness`] or [`Constraint`].
//!
//! A [`Witness`] is a constraint system allocated value that is represented by its identifier
//! and [`Scalar`] value.
//!
//! A [`Constraint`] is a [`Polynomial`] expression represented as a gate of the circuit that will
//! allow computation in the constraint system. It will evaluate to a [`bool`] that is the
//! representation of the result of the gate.
//!
//! Every item of a circuit description contains a mapping to the [`Source`] file that generated it.
//! This will allow the debugger to map the constraint to its original Rust source code.
//!
//! A circuit description format file will contain a preamble with all its witnesses. Provided
//! this, its witness index will reflect its line on the file, facilitating indexing.

mod config;
mod constraint;
mod context;
mod element;
mod encoder;
mod indexed_witness;
mod polynomial;
mod preamble;
mod source;
mod witness;

pub(crate) mod bytes;

use std::fs::{File, OpenOptions};
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::Path;

pub use config::{AtomicConfig, BaseConfig, Config};
pub use constraint::Constraint;
pub use context::{Context, ContextUnit};
pub use element::{Element, FixedText, Scalar};
pub use encoder::Encoder;
pub use indexed_witness::IndexedWitness;
pub use polynomial::Polynomial;
pub use preamble::Preamble;
pub use source::Source;
pub use witness::Witness;

/// A circuit description with a unit backend
pub type CircuitDescriptionUnit = CircuitDescription<()>;

/// A circuit description with a file backend
pub type CircuitDescriptionFile = CircuitDescription<File>;

/// A circuit description file
///
/// Since circuit descriptions are often large, it will perform lazy disk I/O, loading only the
/// required data to satisfy the user operation.
#[derive(Debug)]
pub struct CircuitDescription<S> {
    source: S,
    preamble: Preamble,
}

impl CircuitDescription<File> {
    /// Open a CDF file as read-only.
    pub fn open_read<P>(path: P) -> io::Result<Self>
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
    S: io::Read,
{
    /// Create a new instance of the CDF consumer by reference
    pub fn by_ref(&mut self) -> CircuitDescription<&mut S> {
        CircuitDescription {
            source: self.source.by_ref(),
            preamble: self.preamble,
        }
    }

    /// Create a new context with a referennce to the underlying source
    pub fn context(&mut self) -> Context<&mut S> {
        let ctx = self.by_ref();

        Context::with_cdf(ctx)
    }
}

impl<S> CircuitDescription<S>
where
    S: io::Read + io::Seek,
{
    /// Create a new circuit description instance.
    pub fn from_reader(source: S) -> io::Result<Self> {
        let mut cdf = Self {
            source,
            preamble: Preamble::default(),
        };

        let ctx = &mut cdf.context();

        cdf.preamble = Preamble::try_from_context(&AtomicConfig, ctx)?;

        Ok(cdf)
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

        let config = self.preamble.config;
        let mut ctx = self.context();

        Constraint::try_from_context(&config, &mut ctx)
    }

    /// Attempt to read an indexed witness from the source
    pub fn fetch_witness(&mut self, idx: usize) -> io::Result<Witness> {
        self.preamble
            .witness_offset(idx)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "attempt to fetch invalid witness"))
            .map(|ofs| io::SeekFrom::Start(ofs as u64))
            .and_then(|ofs| self.source.seek(ofs))?;

        let config = self.preamble.config;
        let mut ctx = self.context();

        Witness::try_from_context(&config, &mut ctx)
    }

    /// Attempt to fetch a path from source cache
    pub fn fetch_source(&mut self, idx: usize) -> io::Result<FixedText<{ Source::PATH_LEN }>> {
        let ofs = self.preamble.source_cache_offset(idx);
        let ofs = io::SeekFrom::Start(ofs as u64);

        self.source.seek(ofs)?;

        let config = self.preamble.config;
        let mut ctx = self.context();

        FixedText::try_from_context(&config, &mut ctx)
    }
}

impl<S> CircuitDescription<S> {
    /// Return the inner source
    pub fn into_inner(self) -> S {
        self.source
    }

    /// CDF file metadata
    pub const fn config(&self) -> &Config {
        &self.preamble.config
    }

    /// CDF preamble metadata
    pub const fn preamble(&self) -> &Preamble {
        &self.preamble
    }
}

impl<S> Deref for CircuitDescription<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.source
    }
}

impl<S> DerefMut for CircuitDescription<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.source
    }
}
