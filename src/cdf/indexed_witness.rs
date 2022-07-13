use std::io;

use super::{Element, Preamble, Scalar};

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
}

impl Element for IndexedWitness {
    const LEN: usize = u64::LEN + <Option<u64>>::LEN + Scalar::LEN;

    fn zeroed() -> Self {
        Self::default()
    }

    fn to_buffer(&self, buf: &mut [u8]) {
        let buf = self.index.encode(buf);
        let buf = self.origin.encode(buf);
        let _ = self.value.encode(buf);
    }

    fn try_from_buffer_in_place(&mut self, buf: &[u8]) -> io::Result<()> {
        let buf = self.index.try_decode_in_place(buf)?;
        let buf = self.origin.try_decode_in_place(buf)?;
        let _ = self.value.try_decode_in_place(buf)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.index.validate(preamble)?;
        self.origin.map(|o| o.validate(preamble)).transpose()?;
        self.value.validate(preamble)?;

        if self.index >= preamble.witnesses() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "the provided witness index does not correspond to a valid allocated witness",
            ));
        }

        if let Some(o) = self.origin {
            if o >= preamble.constraints() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "the provided constraint index does not correspond to a valid gate",
                ));
            }
        }

        Ok(())
    }
}

impl io::Write for IndexedWitness {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.try_write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.value.flush()?;

        Ok(())
    }
}

impl io::Read for IndexedWitness {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.try_read(buf)
    }
}
