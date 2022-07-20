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

mod constraint;
mod element;
mod indexed_witness;
mod polynomial;
mod preamble;
mod source;
mod witness;

pub(crate) mod bytes;

use std::borrow::Borrow;
use std::fs::{File, OpenOptions};
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::Path;

pub use constraint::Constraint;
pub use element::{Element, FixedText, Scalar};
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
    /// Create a new circuit description instance.
    pub fn from_reader(mut source: S) -> io::Result<Self> {
        Preamble::try_from_reader(source.by_ref()).map(|preamble| Self { source, preamble })
    }
}

impl<S> CircuitDescription<S>
where
    S: io::Read + io::Seek,
{
    /// Attempt to read an indexed constraint from the source
    pub fn fetch_constraint(&mut self, idx: usize) -> io::Result<Constraint> {
        if idx >= self.preamble.constraints() as usize {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "attempt to fetch invalid constraint",
            ));
        }

        let offset = Preamble::LEN
            + self.preamble.witnesses() as usize * Witness::len(&self.preamble)
            + idx * Constraint::len(&self.preamble);

        let offset = io::SeekFrom::Start(offset as u64);

        self.source.seek(offset)?;

        bytes::try_from_reader(self.source.by_ref(), &self.preamble)
    }

    /// Attempt to read an indexed witness from the source
    pub fn fetch_witness(&mut self, idx: usize) -> io::Result<Witness> {
        if idx >= self.preamble.witnesses() as usize {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "attempt to fetch invalid witness",
            ));
        }

        let offset = Preamble::LEN + idx * Witness::len(&self.preamble);

        let offset = io::SeekFrom::Start(offset as u64);

        self.source.seek(offset)?;

        bytes::try_from_reader(self.source.by_ref(), &self.preamble)
    }
}

impl<S> CircuitDescription<S> {
    /// Return the inner source
    pub fn into_inner(self) -> S {
        self.source
    }

    /// CDF file metadata
    pub const fn preamble(&self) -> &Preamble {
        &self.preamble
    }

    /// Assert the iterator pair is a valid CDF representation.
    ///
    /// The check will reallocate all items of the iterator.
    pub fn into_valid_cdf<W, C>(
        witnesses: W,
        constraints: C,
    ) -> io::Result<(
        Preamble,
        impl Iterator<Item = Witness>,
        impl Iterator<Item = Constraint>,
    )>
    where
        W: Iterator<Item = Witness>,
        C: Iterator<Item = Constraint>,
    {
        let mut witnesses: Vec<Witness> = witnesses.collect();
        let mut constraints: Vec<Constraint> = constraints.collect();

        witnesses.sort_by_key(|w| w.id());
        constraints.sort_by_key(|c| c.id());

        if let Some(w) = witnesses.first() {
            if w.id() != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "the witnesses set doesn't begin with an index 0",
                ));
            }
        }

        if let Some(c) = constraints.first() {
            if c.id() != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "the constraints set doesn't begin with an index 0",
                ));
            }
        }

        let preamble = Preamble::new(witnesses.len() as u64, constraints.len() as u64);

        witnesses.iter().try_for_each(|w| w.validate(&preamble))?;
        constraints.iter().try_for_each(|c| c.validate(&preamble))?;

        witnesses.as_slice().windows(2).try_for_each(|w| {
            let a = w[0].id();
            let b = w[1].id();

            (a.saturating_add(1) == b).then_some(()).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "the provided witnesses does not compute to an ordered dense unique indexed set"
                )
            })
        })?;

        constraints.as_slice().windows(2).try_for_each(|c| {
            let a = c[0].id();
            let b = c[1].id();

            (a.saturating_add(1) == b).then_some(())
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::Other,
                    "the provided constraints does not compute to an ordered dense unique indexed set"
                ))
        })?;

        let witnesses = witnesses.into_iter();
        let constraints = constraints.into_iter();

        Ok((preamble, witnesses, constraints))
    }

    /// Write all items of the provided iterator without checking the consistency of the file.
    ///
    /// The iterator is expected to comply with the rules asserted in [`Self::into_valid_cdf`].
    pub fn write_all_unchecked<W, BW, BC, IW, IC>(
        mut writer: W,
        preamble: Preamble,
        witnesses: IW,
        constraints: IC,
    ) -> io::Result<usize>
    where
        W: io::Write,
        BW: Borrow<Witness>,
        BC: Borrow<Constraint>,
        IW: Iterator<Item = BW>,
        IC: Iterator<Item = BC>,
    {
        let n = preamble.try_to_writer(writer.by_ref())?;

        let n = witnesses
            .map(|w| bytes::try_to_writer(writer.by_ref(), &preamble, w))
            .try_fold::<_, _, io::Result<usize>>(n, |n, x| Ok(n + x?))?;

        let n = constraints
            .map(|c| bytes::try_to_writer(writer.by_ref(), &preamble, c))
            .try_fold::<_, _, io::Result<usize>>(n, |n, x| Ok(n + x?))?;

        Ok(n)
    }

    /// Write all items of the provided iterator, asserting CDF consistency via
    /// [`Self::into_valid_cdf`]
    pub fn write_all<W, IW, IC>(writer: W, witnesses: IW, constraints: IC) -> io::Result<usize>
    where
        W: io::Write,
        IW: Iterator<Item = Witness>,
        IC: Iterator<Item = Constraint>,
    {
        Self::into_valid_cdf(witnesses, constraints).and_then(
            |(preamble, witnesses, constraints)| {
                Self::write_all_unchecked(writer, preamble, witnesses, constraints)
            },
        )
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
