//! Decoding CDF format

mod context;
mod display;

use std::fs::{File, OpenOptions};
use std::path::Path;
use std::{fmt, io};

pub use context::DecoderContext;
pub use display::DecoderDisplay;
use msgpacker::Message;

use crate::{Constraint, DecodableElement, Preamble, Witness};

/// A circuit description file
///
/// Since circuit descriptions are often large, it will perform lazy disk I/O,
/// loading only the required data to satisfy the user operation.
#[derive(Debug, Clone)]
pub struct CircuitDescription<S> {
    preamble: Preamble,
    source_names: Vec<String>,
    source_contents: Vec<String>,
    source: S,
}

impl<S> fmt::Display for CircuitDescription<S>
where
    S: DecoderDisplay,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <String as fmt::Display>::fmt(&self.source.to_string(), f)
    }
}

impl<S> CircuitDescription<S> {
    pub(crate) fn sources(&self) -> impl Iterator<Item = (&str, &str)> {
        self.source_names
            .iter()
            .map(|s| s.as_str())
            .zip(self.source_contents.iter().map(|s| s.as_str()))
    }

    pub(crate) fn context(&mut self) -> (DecoderContext, &mut S) {
        let Self {
            preamble,
            source_names,
            source_contents,
            source,
        } = self;

        let ctx = DecoderContext::new(
            &preamble.config,
            source_names,
            source_contents,
        );

        (ctx, source)
    }

    /// Helper method to return the preamble of the circuit description.
    pub const fn preamble(&self) -> &Preamble {
        &self.preamble
    }

    /// Check if the provided name is contained within the available source
    /// names
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::CircuitDescription;
    ///
    /// let circuit = CircuitDescription::open("../assets/test.cdf")?;
    ///
    /// assert_eq!(circuit.source_name_contains("/home/vlopes/dev/codex/tmp/plonk-dbg-lib/src/main.rs"), true);
    ///
    /// # Ok(()) }
    /// ```
    pub fn source_name_contains(&self, name: &str) -> bool {
        self.source_names.iter().any(|n| n.contains(name))
    }
}

impl CircuitDescription<File> {
    /// Use a path to create a new circuit description. This uses
    /// [`from_reader`] behind.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::CircuitDescription;
    ///
    /// let circuit = CircuitDescription::open("../assets/test.cdf")?;
    ///
    /// assert_eq!(circuit.preamble().constraints, 10);
    ///
    /// # Ok(()) }
    /// ```
    /// [`from_reader`]: CircuitDescription::from_reader
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
    /// Create a new circuit description instance from a readable and seekable
    /// source.
    ///
    /// To load a circuit description from a file, see [`open`]
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::CircuitDescription;
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let circuit = CircuitDescription::from_reader(file)?;
    ///
    /// assert_eq!(circuit.preamble().witnesses, 15);
    /// assert_eq!(circuit.preamble().constraints, 10);
    ///
    /// # Ok(()) }
    /// ```
    /// [`open`]: CircuitDescription::open
    pub fn from_reader(mut source: S) -> io::Result<Self> {
        // reset the cursor
        source.seek(io::SeekFrom::Start(0))?;

        // load the preamble with the base config
        let preamble =
            Preamble::try_from_reader(&DecoderContext::BASE, source.by_ref())?;

        let ofs = preamble.source_cache_offset();
        let ofs = io::SeekFrom::Start(ofs as u64);
        source.seek(ofs)?;

        let source_names = Message::unpack(source.by_ref())?;
        let source_contents = Message::unpack(source.by_ref())?;

        let (source_names, source_contents) =
            match (source_names, source_contents) {
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

    /// Attempt to read an indexed constraint from the source.
    ///
    /// The idx argument is the index of the constraint you want to fetch.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::CircuitDescription;
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut circuit = CircuitDescription::from_reader(file)?;
    /// let constraint = circuit.fetch_constraint(1)?;
    ///
    /// assert_eq!(constraint.id(), 1);
    ///
    /// # Ok(()) }
    /// ```
    pub fn fetch_constraint(&mut self, idx: usize) -> io::Result<Constraint> {
        self.preamble
            .constraint_offset(idx)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "attempt to fetch invalid constraint",
                )
            })
            .map(|ofs| io::SeekFrom::Start(ofs as u64))
            .and_then(|ofs| self.source.seek(ofs))?;

        let (ctx, source) = self.context();

        Constraint::try_from_reader(&ctx, source)
    }

    /// Attempt to read an indexed witness from the source.
    ///
    /// The idx argument is the index of the witness you want to fetch.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::CircuitDescription;
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut circuit = CircuitDescription::from_reader(file)?;
    /// let witness = circuit.fetch_witness(1)?;
    ///
    /// assert_eq!(witness.id(), 1);
    ///
    /// # Ok(()) }
    /// ```
    pub fn fetch_witness(&mut self, idx: usize) -> io::Result<Witness> {
        self.preamble
            .witness_offset(idx)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "attempt to fetch invalid witness",
                )
            })
            .map(|ofs| io::SeekFrom::Start(ofs as u64))
            .and_then(|ofs| self.source.seek(ofs))?;

        let (ctx, source) = self.context();

        Witness::try_from_reader(&ctx, source)
    }
}
