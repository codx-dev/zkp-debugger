//! CDF encoding/encoding configuration

use core::mem;
use std::io;

use serde::{Deserialize, Serialize};
use toml_base_config::BaseConfig;

use crate::{
    DecodableElement, DecoderContext, Element, EncodableElement,
    EncoderContext, Preamble,
};

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
