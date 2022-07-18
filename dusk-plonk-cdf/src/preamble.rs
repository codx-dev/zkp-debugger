use core::mem;
use std::io;

use super::Element;

/// Metadata information of the CDF file
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Preamble {
    witnesses: u64,
    constraints: u64,
}

impl Preamble {
    /// Serialized lenght
    pub const LEN: usize = 2 * mem::size_of::<u64>();

    /// Zeroed preamble
    pub const ZEROED: Self = Self::new(1, 0);

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

    /// Serialize the preamble into a buffer
    pub fn to_buffer(&self, buf: &mut [u8]) {
        let buf = self.witnesses.encode(self, buf);
        let _ = self.constraints.encode(self, buf);
    }

    /// Attempt to decode the preamble from a buffer
    pub fn try_from_buffer_in_place(&mut self, buf: &[u8]) -> io::Result<()> {
        let mut witnesses = 0;
        let mut constraints = 0;

        let buf = witnesses.try_decode_in_place(self, buf)?;
        let _ = constraints.try_decode_in_place(self, buf)?;

        self.witnesses = witnesses;
        self.constraints = constraints;

        Ok(())
    }

    /// Send the bytes representation of an element to a writer
    pub fn try_to_writer<W>(&self, mut writer: W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut bytes = vec![0u8; Self::LEN];

        self.to_buffer(&mut bytes);

        writer.write(&bytes)
    }

    /// Attempt to create a preamble from a reader
    pub fn try_from_reader<R>(mut reader: R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut bytes = vec![0u8; Self::LEN];
        let _ = reader.read(&mut bytes)?;

        let mut preamble = Self::ZEROED;

        preamble.try_from_buffer_in_place(&bytes)?;

        Ok(preamble)
    }
}
