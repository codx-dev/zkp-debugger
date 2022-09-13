use core::mem;
use std::path::{Path, PathBuf};
use std::{fs, io};

use serde::{Deserialize, Serialize};

use crate::{
    DecodableElement, DecoderContext, Element, EncodableElement, EncoderContext, Preamble,
};

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
        Self::path()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "unable to define configuration path")
            })
            .and_then(Self::load_path)
    }

    /// Load a config file from a given path
    fn load_path<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

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
    fn len(ctx: &Config) -> usize {
        bool::len(ctx)
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.zeroed_scalar_values.validate(preamble)?;

        Ok(())
    }
}

impl EncodableElement for Config {
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]) {
        let _ = self.zeroed_scalar_values.encode(ctx, buf);
    }
}

impl DecodableElement for Config {
    fn try_from_buffer_in_place<'a, 'b>(
        &'a mut self,
        ctx: &DecoderContext<'a>,
        buf: &'b [u8],
    ) -> io::Result<()> {
        Self::validate_buffer(ctx.config(), buf)?;

        let _ = self.zeroed_scalar_values.try_decode_in_place(ctx, buf)?;

        Ok(())
    }
}

#[test]
fn base_config_load_works() {
    let dir = tempdir::TempDir::new("base_config").expect("failed to create temp dir");
    let path = dir.path().join("config.toml");

    let config = Config::load_path(&path).expect("failed to load config from path");

    assert_eq!(config, Config::default());

    let config = Config::load_path(&path).expect("failed to read config from path");
    let c = *Config::default().with_zeroed_scalar_values(Config::default().zeroed_scalar_values);

    assert_eq!(config, c);

    Config::load().expect("failed to load default config");
}
