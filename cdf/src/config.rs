//! CDF encoding/encoding configuration

use core::mem;
use std::path::{Path, PathBuf};
use std::{fs, io};

use serde::{Deserialize, Serialize};

use crate::{
    DecodableElement, DecoderContext, Element, EncodableElement,
    EncoderContext, Preamble,
};

/// Base configuration schema. Maintains a config.toml for configs. Handles the
/// reading/writing of config.toml. The config file is stored in the
/// [`dirs::config_dir()`] and the full relative path can be obtained via
/// [`BaseConfig::path`].
///
/// **See**: [`Config`] implements this already.
pub trait BaseConfig:
    Sized + Default + Serialize + for<'a> Deserialize<'a>
{
    /// The Package name is usually `env!("CARGO_PKG_NAME")`. This is the name
    /// of the folder inside the config dir. We store the config.toml inside
    /// this folder.
    ///
    /// Calling [`BaseConfig::load`] will create the config file at
    /// [`BaseConfig::path`].
    const PACKAGE: &'static str;

    /// Compute path for the `config.toml`.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use dusk_cdf::BaseConfig;
    /// use std::path::{Path, Component};
    /// use std::ffi::OsStr;
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// struct T  {}
    ///
    /// impl Default for T {
    ///     fn default() -> Self {
    ///         T {}
    ///     }
    /// }
    ///
    /// impl BaseConfig for T {
    ///     const PACKAGE: &'static str = "Lisa";    
    /// }
    ///
    /// let path = T::path().unwrap();
    /// let mut components = path.components().rev(); // look at the last parts
    ///
    /// assert_eq!(components.next(), Some(Component::Normal(OsStr::new("config.toml"))));
    /// assert_eq!(components.next(), Some(Component::Normal(OsStr::new("Lisa"))));
    /// ```
    fn path() -> Option<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join(Self::PACKAGE))
            .map(|p| p.join("config.toml"))
    }

    /// Serialize the toml if config.toml exists, else create a config.toml at
    /// the path returned by [`BaseConfig::path`].
    ///
    /// The contents for the created config.toml is obtained by
    /// `BaseConfig::default()` implementation
    fn load() -> io::Result<Self> {
        Self::path()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "unable to define configuration path",
                )
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
                .unwrap_or_else(|e| {
                    eprintln!("failed to serialize config file: {}", e)
                });

            return Ok(config);
        }

        let contents = fs::read_to_string(path)?;

        toml::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

/// Configuration parameters for encoding and decoding.
///
/// See [`BaseConfig`] for context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Flag to zero skip scalar values during encoding, and zero them during
    /// decoding.
    pub zeroed_scalar_values: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Config {
    /// Serialized length.
    pub const LEN: usize = mem::size_of::<bool>();

    /// Store a const default with [`zeroed_scalar_values`] set to false.
    ///
    /// [`zeroed_scalar_values`]: structfield.zeroed_scalar_values
    pub const DEFAULT: Self = Self {
        zeroed_scalar_values: false,
    };

    /// If true, then don't store the scalar values and deserialize them as zero
    /// in [`Scalar`](struct.Scalar.html).
    pub fn with_zeroed_scalar_values(
        &mut self,
        zeroed_scalar_values: bool,
    ) -> &mut Self {
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
    let dir = tempdir::TempDir::new("base_config")
        .expect("failed to create temp dir");
    let path = dir.path().join("config.toml");

    let config =
        Config::load_path(&path).expect("failed to load config from path");

    assert_eq!(config, Config::default());

    let config =
        Config::load_path(&path).expect("failed to read config from path");
    let c = *Config::default()
        .with_zeroed_scalar_values(Config::default().zeroed_scalar_values);

    assert_eq!(config, c);

    Config::load().expect("failed to load default config");
}
