use std::io;

use crate::{AtomicConfig, Config, Element, Preamble, Scalar, Source};

/// Witness allocation representation
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Witness {
    id: u64,
    value: Scalar,
    source: Source,
}

impl Witness {
    /// Create a new witness
    pub const fn new(id: u64, value: Scalar, source: Source) -> Self {
        Self { id, value, source }
    }

    /// Id of the witness in the constraint system
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Value of the witness in the constraint system
    pub const fn value(&self) -> &Scalar {
        &self.value
    }

    /// Source reference of the witness
    pub const fn source(&self) -> &Source {
        &self.source
    }
}

impl Element for Witness {
    type Config = Config;

    fn zeroed() -> Self {
        Self::default()
    }

    fn len(config: &Self::Config) -> usize {
        u64::len(&AtomicConfig) + Scalar::len(config) + Source::len(config)
    }

    fn to_buffer(&self, config: &Self::Config, buf: &mut [u8]) {
        let buf = self.id.encode(&AtomicConfig, buf);
        let buf = self.value.encode(config, buf);
        let _ = self.source.encode(config, buf);
    }

    fn try_from_buffer_in_place(&mut self, config: &Self::Config, buf: &[u8]) -> io::Result<()> {
        let buf = self.id.try_decode_in_place(&AtomicConfig, buf)?;
        let buf = self.value.try_decode_in_place(config, buf)?;
        let _ = self.source.try_decode_in_place(config, buf)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.id.validate(preamble)?;
        self.value.validate(preamble)?;
        self.source.validate(preamble)?;

        Ok(())
    }
}
