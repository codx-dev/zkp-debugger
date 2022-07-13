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
}

impl Element for Witness {
    const LEN: usize = u64::LEN + Scalar::LEN + Source::LEN;

    fn zeroed() -> Self {
        Self::default()
    }

    fn to_buffer(&self, buf: &mut [u8]) {
        let buf = self.id.encode(buf);
        let buf = self.value.encode(buf);
        let _ = self.source.encode(buf);
    }

    fn try_from_buffer_in_place(&mut self, buf: &[u8]) -> io::Result<()> {
        let buf = self.id.try_decode_in_place(buf)?;
        let buf = self.value.try_decode_in_place(buf)?;
        let _ = self.source.try_decode_in_place(buf)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.id.validate(preamble)?;
        self.value.validate(preamble)?;
        self.source.validate(preamble)?;

        Ok(())
    }
}

impl io::Write for Witness {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.try_write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.value.flush()?;
        self.source.flush()?;

        Ok(())
    }
}

impl io::Read for Witness {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.try_read(buf)
    }
}
