use std::io;

use super::{Element, Preamble, Scalar, Source};

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
    fn zeroed() -> Self {
        Self::default()
    }

    fn len(preamble: &Preamble) -> usize {
        u64::len(preamble) + Scalar::len(preamble) + Source::len(preamble)
    }

    fn to_buffer(&self, preamble: &Preamble, buf: &mut [u8]) {
        let buf = self.id.encode(preamble, buf);
        let buf = self.value.encode(preamble, buf);
        let _ = self.source.encode(preamble, buf);
    }

    fn try_from_buffer_in_place(&mut self, preamble: &Preamble, buf: &[u8]) -> io::Result<()> {
        let buf = self.id.try_decode_in_place(preamble, buf)?;
        let buf = self.value.try_decode_in_place(preamble, buf)?;
        let _ = self.source.try_decode_in_place(preamble, buf)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.id.validate(preamble)?;
        self.value.validate(preamble)?;
        self.source.validate(preamble)?;

        Ok(())
    }
}
