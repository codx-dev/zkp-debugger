use std::io;

use crate::{Config, Element, Preamble, Scalar, AtomicConfig};

/// Representation of an indexed witness.
///
/// Its index is the dense representation of the constraint system. Since CDF files will store
/// witnesses first, it will be line (starting at zero) of the file.
///
/// Its origin will be the constraint that created this witness. Its `None` when the witness is
/// referenced on the same constraint it was created.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IndexedWitness {
    index: u64,
    origin: Option<u64>,
    value: Scalar,
}

impl IndexedWitness {
    /// Create a new indexed witness
    pub const fn new(index: u64, origin: Option<u64>, value: Scalar) -> Self {
        Self {
            index,
            origin,
            value,
        }
    }

    /// Witness index
    pub const fn index(&self) -> u64 {
        self.index
    }

    /// Witness constraint originator
    pub const fn origin(&self) -> &Option<u64> {
        &self.origin
    }

    /// Witness value
    pub const fn value(&self) -> &Scalar {
        &self.value
    }
}

impl Element for IndexedWitness {
    type Config = Config;

    fn zeroed() -> Self {
        Self::default()
    }

    fn len(config: &Self::Config) -> usize {
        u64::len(&AtomicConfig) + <Option<u64>>::len(&AtomicConfig) + Scalar::len(config)
    }

    fn to_buffer(&self, config: &Self::Config, buf: &mut [u8]) {
        let buf = self.index.encode(&AtomicConfig, buf);
        let buf = self.origin.encode(&AtomicConfig, buf);
        let _ = self.value.encode(config, buf);
    }

    fn try_from_buffer_in_place(&mut self, config: &Self::Config, buf: &[u8]) -> io::Result<()> {
        let buf = self.index.try_decode_in_place(&AtomicConfig, buf)?;
        let buf = self.origin.try_decode_in_place(&AtomicConfig, buf)?;
        let _ = self.value.try_decode_in_place(config, buf)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.index.validate(preamble)?;
        self.origin.map(|o| o.validate(preamble)).transpose()?;
        self.value.validate(preamble)?;

        if self.index >= preamble.witnesses as u64 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "the provided witness index does not correspond to a valid allocated witness",
            ));
        }

        if let Some(o) = self.origin {
            if o >= preamble.constraints as u64 {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "the provided constraint index does not correspond to a valid gate",
                ));
            }
        }

        Ok(())
    }
}
