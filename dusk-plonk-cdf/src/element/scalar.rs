use std::io;
use std::ops::{Deref, DerefMut};

use super::{Element, Preamble};

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
    fn zeroed() -> Self {
        Self::default()
    }

    fn len(_preamble: &Preamble) -> usize {
        Self::LEN
    }

    fn to_buffer(&self, _preamble: &Preamble, buf: &mut [u8]) {
        let buf = &mut buf[..Self::LEN];

        buf.copy_from_slice(&self.scalar);
    }

    fn try_from_buffer_in_place(&mut self, _preamble: &Preamble, buf: &[u8]) -> io::Result<()> {
        self.scalar.copy_from_slice(&buf[..Self::LEN]);

        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}
