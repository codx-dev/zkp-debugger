use core::mem;
use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

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

/// Base configuration schema
pub trait BaseConfig: Sized + Default + Serialize + for<'a> Deserialize<'a> {
    /// Package name (e.g. `CARGO_PKG_NAME`)
    const PACKAGE: &'static str;

    /// Path of the serialized configuration
    fn path() -> Option<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join(Self::PACKAGE))
            .map(|p| p.join("config.toml"))
    }

    /// Load a config instance from the config dir
    fn load() -> io::Result<Self> {
        let path = Self::path().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "unable to define configuration path")
        })?;

        if !path.exists() {
            let config = Self::default();

            // config serialization is optional
            path.parent()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        "unable to fetch parent dir of config file",
                    )
                })
                .and_then(fs::create_dir_all)
                .and_then(|_| {
                    toml::to_string_pretty(&config)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                })
                .and_then(|contents| fs::write(path, contents))
                .unwrap_or_else(|e| eprintln!("failed to serialize config file: {}", e));

            return Ok(config);
        }

        let contents = fs::read_to_string(path)?;

        toml::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

/// Configuration parameters for encoding and decoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

    /// Set the flag to cache the source path
    pub fn with_zeroed_scalar_values(&mut self, zeroed_scalar_values: bool) -> &mut Self {
        self.zeroed_scalar_values = zeroed_scalar_values;
        self
    }
}

impl BaseConfig for Config {
    const PACKAGE: &'static str = env!("CARGO_PKG_NAME");
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
        Config::default()
            .with_zeroed_scalar_values(true)
            .zeroed_scalar_values
    );

    assert!(
        !Config::default()
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
