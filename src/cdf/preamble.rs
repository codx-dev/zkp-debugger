use std::io;

use super::Element;

/// Metadata information of the CDF file
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Preamble {
    witnesses: u64,
    constraints: u64,
}

impl Preamble {
    /// Create a new preamble instance.
    pub const fn new(witnesses: u64, constraints: u64) -> Self {
        // Empty witness set can't produce a valid PLONK circuit since the first witness is
        // reserved per protocol
        debug_assert!(witnesses > 0);

        Self {
            witnesses,
            constraints,
        }
    }

    /// Witnesses count
    pub const fn witnesses(&self) -> u64 {
        self.witnesses
    }

    /// Constraints count
    pub const fn constraints(&self) -> u64 {
        self.constraints
    }
}

impl Element for Preamble {
    const LEN: usize = 16;

    fn zeroed() -> Self {
        Self::default()
    }

    fn to_buffer(&self, buf: &mut [u8]) {
        let buf = self.witnesses.encode(buf);
        let _ = self.constraints.encode(buf);
    }

    fn try_from_buffer_in_place(&mut self, buf: &[u8]) -> io::Result<()> {
        let buf = self.witnesses.try_decode_in_place(buf)?;
        let _ = self.constraints.try_decode_in_place(buf)?;

        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

impl io::Write for Preamble {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.try_write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Read for Preamble {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.try_read(buf)
    }
}
