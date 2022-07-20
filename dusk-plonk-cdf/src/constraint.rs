use std::io;

use crate::{AtomicConfig, Config, Element, Polynomial, Preamble, Scalar, Source};

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
    type Config = Config;

    fn zeroed() -> Self {
        Self::default()
    }

    fn len(config: &Self::Config) -> usize {
        u64::len(&AtomicConfig)
            + Scalar::len(config)
            + Polynomial::len(config)
            + Source::len(config)
    }

    fn to_buffer(&self, config: &Self::Config, buf: &mut [u8]) {
        let buf = self.id.encode(&AtomicConfig, buf);
        let buf = self.polynomial.encode(config, buf);
        let _ = self.source.encode(config, buf);
    }

    fn try_from_buffer_in_place(&mut self, config: &Self::Config, buf: &[u8]) -> io::Result<()> {
        let buf = self.id.try_decode_in_place(&AtomicConfig, buf)?;
        let buf = self.polynomial.try_decode_in_place(config, buf)?;
        let _ = self.source.try_decode_in_place(config, buf)?;

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.id.validate(preamble)?;
        self.polynomial.validate(preamble)?;
        self.source.validate(preamble)?;

        Ok(())
    }
}

#[test]
fn validate_works() {
    Constraint::zeroed()
        .validate(&Default::default())
        .expect("default config validate should pass");
}
