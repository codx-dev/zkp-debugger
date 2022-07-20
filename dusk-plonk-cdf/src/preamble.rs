use core::mem;
use std::io;

use crate::{AtomicConfig, Config, Constraint, Element, Witness};

/// Metadata information of the CDF file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preamble {
    /// Witnesses count
    pub witnesses: usize,
    /// Constraints count
    pub constraints: usize,
    /// Configuration parameters for encoding and decoding
    pub config: Config,
}

impl Default for Preamble {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Preamble {
    /// Serialized length
    pub const LEN: usize = 2 * mem::size_of::<usize>() + Config::LEN;

    /// Default preamble
    pub const DEFAULT: Self = Self {
        witnesses: 1,
        constraints: 0,
        config: Config::DEFAULT,
    };

    /// Create a new preamble instance.
    pub const fn new() -> Self {
        Self::DEFAULT
    }

    /// Set the witnesses count
    pub fn with_witnesses(&mut self, witnesses: usize) -> &mut Self {
        // Empty witness set can't produce a valid PLONK circuit since the first witness is
        // reserved per protocol
        debug_assert!(witnesses > 0);

        self.witnesses = witnesses;
        self
    }

    /// Set the constraints count
    pub fn with_constraints(&mut self, constraints: usize) -> &mut Self {
        self.constraints = constraints;
        self
    }

    /// Set the configuration parameters
    pub fn with_config(&mut self, config: Config) -> &mut Self {
        self.config = config;
        self
    }

    /// Calculate the total length of the CDF output
    pub fn total_len(&self) -> usize {
        Self::LEN
            + self.witnesses * Witness::len(&self.config)
            + self.constraints * Constraint::len(&self.config)
    }

    /// Witness offset in CDF, from an index
    pub fn witness_offset(&self, idx: usize) -> Option<usize> {
        (idx < self.witnesses).then(|| Self::LEN + idx * Witness::len(&self.config))
    }

    /// Constraint offset in CDF, from an index
    pub fn constraint_offset(&self, idx: usize) -> Option<usize> {
        (idx < self.constraints).then(|| {
            Self::LEN
                + self.witnesses * Witness::len(&self.config)
                + idx * Constraint::len(&self.config)
        })
    }
}

impl Element for Preamble {
    type Config = AtomicConfig;

    fn zeroed() -> Self {
        Self::DEFAULT
    }

    fn len(_config: &Self::Config) -> usize {
        Self::LEN
    }

    fn to_buffer(&self, _config: &Self::Config, buf: &mut [u8]) {
        let buf = self.witnesses.encode(&AtomicConfig, buf);
        let buf = self.constraints.encode(&AtomicConfig, buf);
        let _ = self.config.encode(&AtomicConfig, buf);
    }

    fn try_from_buffer_in_place(&mut self, _config: &Self::Config, buf: &[u8]) -> io::Result<()> {
        let buf = self.witnesses.try_decode_in_place(&AtomicConfig, buf)?;
        let buf = self.constraints.try_decode_in_place(&AtomicConfig, buf)?;
        let _ = self.config.try_decode_in_place(&AtomicConfig, buf)?;

        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        if self.witnesses == 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "witness count can't be zero in PLONK protocol",
            ));
        }

        Ok(())
    }
}
