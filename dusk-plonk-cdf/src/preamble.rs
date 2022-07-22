use core::mem;
use std::io;

use crate::{AtomicConfig, Config, Constraint, Context, ContextUnit, Element, Source, Witness};

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

    /// Cache starting position
    pub fn source_cache_offset(&self, idx: usize) -> usize {
        Self::LEN
            + self.witnesses * Witness::len(&self.config)
            + self.constraints * Constraint::len(&self.config)
            + idx * Source::PATH_LEN as usize
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

    fn to_buffer(&self, _config: &Self::Config, context: &mut ContextUnit, buf: &mut [u8]) {
        let buf = self.witnesses.encode(&AtomicConfig, context, buf);
        let buf = self.constraints.encode(&AtomicConfig, context, buf);
        let _ = self.config.encode(&AtomicConfig, context, buf);
    }

    fn try_from_buffer_in_place<S>(
        &mut self,
        _config: &Self::Config,
        context: &mut Context<S>,
        buf: &[u8],
    ) -> io::Result<()>
    where
        S: io::Read + io::Seek,
    {
        let buf = self
            .witnesses
            .try_decode_in_place(&AtomicConfig, context, buf)?;
        let buf = self
            .constraints
            .try_decode_in_place(&AtomicConfig, context, buf)?;
        let _ = self
            .config
            .try_decode_in_place(&AtomicConfig, context, buf)?;

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

#[test]
fn validate_works() {
    Preamble::zeroed()
        .validate(&Default::default())
        .expect("default config validate should pass");

    Preamble::new()
        .with_witnesses(0)
        .validate(&Default::default())
        .expect_err("zeroed witness count isn't a valid circuit");
}
