use std::io;
use std::ops::{Deref, DerefMut};

use serde::Serialize;

use crate::{
    Config, DecodableElement, DecoderContext, Element, EncodableElement,
    EncoderContext, Preamble,
};

/// Scalar field representation with up to 256 bits.
///
/// This is agnostic to the curve choice and no canonical encoding assumption is
/// involved.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize,
)]
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
    fn len(ctx: &Config) -> usize {
        if ctx.zeroed_scalar_values {
            0
        } else {
            Self::LEN
        }
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

impl EncodableElement for Scalar {
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]) {
        if !ctx.config().zeroed_scalar_values {
            let buf = &mut buf[..Self::LEN];

            buf.copy_from_slice(&self.scalar);
        }
    }
}

impl DecodableElement for Scalar {
    fn try_from_buffer_in_place<'b>(
        &mut self,
        ctx: &DecoderContext,
        buf: &'b [u8],
    ) -> io::Result<()> {
        Self::validate_buffer(ctx.config(), buf)?;

        if !ctx.config().zeroed_scalar_values {
            self.scalar.copy_from_slice(&buf[..Self::LEN]);
        }

        Ok(())
    }
}
