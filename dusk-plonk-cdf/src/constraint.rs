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

    /// Source file that originated the constraint
    pub const fn source(&self) -> &Source {
        &self.source
    }

    /// Check if the polynomial evaluation is ok
    pub const fn is_ok(&self) -> bool {
        self.polynomial.is_ok()
    }
}

impl Element for Constraint {
    fn zeroed() -> Self {
        Self::default()
    }

    fn len(preamble: &Preamble) -> usize {
        u64::len(preamble)
            + Scalar::len(preamble)
            + Polynomial::len(preamble)
            + Source::len(preamble)
    }

    fn to_buffer(&self, preamble: &Preamble, buf: &mut [u8]) {
        let buf = self.id.encode(preamble, buf);
        let buf = self.polynomial.encode(preamble, buf);
        let _ = self.source.encode(preamble, buf);
    }

    fn try_from_buffer_in_place(&mut self, preamble: &Preamble, buf: &[u8]) -> io::Result<()> {
        let buf = self.id.try_decode_in_place(preamble, buf)?;
        let buf = self.polynomial.try_decode_in_place(preamble, buf)?;
        let _ = self.source.try_decode_in_place(preamble, buf)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.id.validate(preamble)?;
        self.polynomial.validate(preamble)?;
        self.source.validate(preamble)?;

        Ok(())
    }
}
