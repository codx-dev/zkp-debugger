use std::io;
use std::ops::{Deref, DerefMut};

use crate::{Config, Element, Preamble};

/// Scalar field representation with up to 256 bits.
///
/// This is agnostic to the curve choice and no canonical encoding assumption is involved.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scalar {
    scalar: [u8; Self::LEN],
}

impl Scalar {
    /// Fixed serialized length
    pub const LEN: usize = 32;
}

impl From<[u8; Scalar::LEN]> for Scalar {
    fn from(scalar: [u8; Self::LEN]) -> Self {
        Self { scalar }
    }
}

impl AsRef<[u8]> for Scalar {
    fn as_ref(&self) -> &[u8] {
        &self.scalar
    }
}

impl Deref for Scalar {
    type Target = [u8; Self::LEN];

    fn deref(&self) -> &Self::Target {
        &self.scalar
    }
}

impl DerefMut for Scalar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scalar
    }
}

impl Element for Scalar {
    type Config = Config;

    fn zeroed() -> Self {
        Self::default()
    }

    fn len(config: &Self::Config) -> usize {
        if config.zeroed_scalar_values {
            0
        } else {
            Self::LEN
        }
    }

    fn to_buffer(&self, config: &Self::Config, buf: &mut [u8]) {
        if !config.zeroed_scalar_values {
            let buf = &mut buf[..Self::LEN];

            buf.copy_from_slice(&self.scalar);
        }
    }

    fn try_from_buffer_in_place(&mut self, config: &Self::Config, buf: &[u8]) -> io::Result<()> {
        if !config.zeroed_scalar_values {
            self.scalar.copy_from_slice(&buf[..Self::LEN]);
        }

        Ok(())
    }

    fn validate(&self, _config: &Preamble) -> io::Result<()> {
        Ok(())
    }
}
