use std::io;

use crate::{AtomicConfig, Config, Context, ContextUnit, Element, Preamble, Scalar, Source};

/// Witness allocation representation
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Witness {
    id: usize,
    value: Scalar,
    source: Source,
}

impl Witness {
    /// Create a new witness
    pub const fn new(id: usize, value: Scalar, source: Source) -> Self {
        Self { id, value, source }
    }

    /// Id of the witness in the constraint system
    pub const fn id(&self) -> usize {
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
        usize::len(&AtomicConfig) + Scalar::len(config) + Source::len(config)
    }

    fn to_buffer(&self, config: &Self::Config, context: &mut ContextUnit, buf: &mut [u8]) {
        let buf = self.id.encode(&AtomicConfig, context, buf);
        let buf = self.value.encode(config, context, buf);
        let _ = self.source.encode(config, context, buf);
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
        let buf = self.id.try_decode_in_place(&AtomicConfig, context, buf)?;
        let buf = self.value.try_decode_in_place(config, context, buf)?;
        let _ = self.source.try_decode_in_place(config, context, buf)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.id.validate(preamble)?;
        self.value.validate(preamble)?;
        self.source.validate(preamble)?;

        Ok(())
    }
}
