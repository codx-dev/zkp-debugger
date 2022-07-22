use std::io;

use crate::{AtomicConfig, Config, Context, ContextUnit, Element, Preamble, Scalar};

/// Representation of an indexed witness.
///
/// Its index is the dense representation of the constraint system. Since CDF files will store
/// witnesses first, it will be line (starting at zero) of the file.
///
/// Its origin will be the constraint that created this witness. Its `None` when the witness is
/// referenced on the same constraint it was created.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IndexedWitness {
    index: usize,
    origin: Option<usize>,
    value: Scalar,
}

impl IndexedWitness {
    /// Create a new indexed witness
    pub const fn new(index: usize, origin: Option<usize>, value: Scalar) -> Self {
        Self {
            index,
            origin,
            value,
        }
    }

    /// Witness index
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Witness constraint originator
    pub const fn origin(&self) -> &Option<usize> {
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
        usize::len(&AtomicConfig) + <Option<usize>>::len(&AtomicConfig) + Scalar::len(config)
    }

    fn to_buffer(&self, config: &Self::Config, context: &mut ContextUnit, buf: &mut [u8]) {
        let buf = self.index.encode(&AtomicConfig, context, buf);
        let buf = self.origin.encode(&AtomicConfig, context, buf);
        let _ = self.value.encode(config, context, buf);
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
        let buf = self
            .index
            .try_decode_in_place(&AtomicConfig, context, buf)?;
        let buf = self
            .origin
            .try_decode_in_place(&AtomicConfig, context, buf)?;
        let _ = self.value.try_decode_in_place(config, context, buf)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.index.validate(preamble)?;
        self.origin.map(|o| o.validate(preamble)).transpose()?;
        self.value.validate(preamble)?;

        if self.index >= preamble.witnesses {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "the provided witness index does not correspond to a valid allocated witness",
            ));
        }

        if let Some(o) = self.origin {
            if o >= preamble.constraints {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "the provided constraint index does not correspond to a valid gate",
                ));
            }
        }

        Ok(())
    }
}
