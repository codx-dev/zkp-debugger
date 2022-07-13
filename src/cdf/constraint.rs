use std::io;

use super::{Element, Polynomial, Preamble, Scalar, Source};

/// Constraint gate of a circuit
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Constraint {
    id: u64,
    polynomial: Polynomial,
    source: Source,
}

impl Constraint {
    /// Generate a new constraint
    pub const fn new(id: u64, polynomial: Polynomial, source: Source) -> Self {
        Self {
            id,
            polynomial,
            source,
        }
    }

    /// Id of the gate in the constraint system
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Polynomial of the gate in the constraint system
    pub const fn polynomial(&self) -> &Polynomial {
        &self.polynomial
    }
}

impl Element for Constraint {
    const LEN: usize = u64::LEN + Scalar::LEN + Polynomial::LEN + Source::LEN;

    fn zeroed() -> Self {
        Self::default()
    }

    fn to_buffer(&self, buf: &mut [u8]) {
        let buf = self.id.encode(buf);
        let buf = self.polynomial.encode(buf);
        let _ = self.source.encode(buf);
    }

    fn try_from_buffer_in_place(&mut self, buf: &[u8]) -> io::Result<()> {
        let buf = self.id.try_decode_in_place(buf)?;
        let buf = self.polynomial.try_decode_in_place(buf)?;
        let _ = self.source.try_decode_in_place(buf)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.id.validate(preamble)?;
        self.polynomial.validate(preamble)?;
        self.source.validate(preamble)?;

        Ok(())
    }
}

impl io::Write for Constraint {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.try_write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.polynomial.flush()?;
        self.source.flush()?;

        Ok(())
    }
}

impl io::Read for Constraint {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.try_read(buf)
    }
}
