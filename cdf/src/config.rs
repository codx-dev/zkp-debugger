use core::mem;
use std::io;

use crate::{Context, ContextUnit, Element, Preamble};

/// Empty config set for atomic serialization that is not parametrizable
pub struct AtomicConfig;

impl Default for AtomicConfig {
    fn default() -> Self {
        AtomicConfig
    }
}

impl From<&Config> for AtomicConfig {
    fn from(_config: &Config) -> Self {
        AtomicConfig
    }
}

impl From<&Config> for Config {
    fn from(config: &Config) -> Self {
        *config
    }
}

/// Configuration parameters for encoding and decoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// Flag to zero skip scalar values during encoding, and zero them during decoding
    pub zeroed_scalar_values: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Config {
    /// Serialized length
    pub const LEN: usize = mem::size_of::<bool>();

    /// Default value as constant
    pub const DEFAULT: Self = Self {
        zeroed_scalar_values: false,
    };

    /// Create a new config instance.
    pub const fn new() -> Self {
        Self::DEFAULT
    }

    /// Set the flag to cache the source path
    pub fn with_zeroed_scalar_values(&mut self, zeroed_scalar_values: bool) -> &mut Self {
        self.zeroed_scalar_values = zeroed_scalar_values;
        self
    }
}

impl Element for Config {
    type Config = AtomicConfig;

    fn zeroed() -> Self {
        Self::DEFAULT
    }

    fn len(_config: &Self::Config) -> usize {
        Self::LEN
    }

    fn to_buffer(&self, _config: &Self::Config, context: &mut ContextUnit, buf: &mut [u8]) {
        let _ = self
            .zeroed_scalar_values
            .encode(&AtomicConfig, context, buf);
    }

    fn try_from_buffer_in_place<S>(
        &mut self,
        config: &Self::Config,
        context: &mut Context<S>,
        buf: &[u8],
    ) -> io::Result<()>
    where
        S: io::Read + io::Seek,
    {
        Self::validate_buffer_len(config, buf.len())?;

        let _ = self
            .zeroed_scalar_values
            .try_decode_in_place(&AtomicConfig, context, buf)?;

        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn builder_functions_works() {
    assert!(
        Config::new()
            .with_zeroed_scalar_values(true)
            .zeroed_scalar_values
    );

    assert!(
        !Config::new()
            .with_zeroed_scalar_values(false)
            .zeroed_scalar_values
    );
}

#[test]
fn zeroed_works() {
    assert_eq!(Config::zeroed(), Config::DEFAULT)
}

#[test]
fn atomic_config_has_default() {
    AtomicConfig::default();
}

#[test]
fn validate_works() {
    Config::default()
        .validate(&Default::default())
        .expect("default config validate should pass");
}
