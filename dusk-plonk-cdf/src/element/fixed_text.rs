use std::ops::Deref;
use std::{fmt, io};

use super::{Element, Preamble};

/// Text representation with fixed `N` bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedText<const N: u16>(String);

impl<const N: u16> fmt::Display for FixedText<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<const N: u16> Deref for FixedText<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// DerefMut is intentionally not provided so the user can't manipulate the inner string into
// invalid states

impl<const N: u16> From<String> for FixedText<N> {
    fn from(mut s: String) -> Self {
        assert!(u16::LEN <= N as usize);

        s.truncate(N as usize - u16::LEN);

        Self(s)
    }
}

impl<const N: u16> From<FixedText<N>> for String {
    fn from(t: FixedText<N>) -> Self {
        t.0
    }
}

impl<const N: u16> Element for FixedText<N> {
    const LEN: usize = N as usize;

    fn zeroed() -> Self {
        Self::default()
    }

    fn to_buffer(&self, buf: &mut [u8]) {
        let bytes = self.0.as_bytes();

        let buf = (bytes.len() as u16).encode(buf);

        let _ = Self::encode_bytes(bytes, buf);
    }

    fn try_from_buffer_in_place(&mut self, buf: &[u8]) -> io::Result<()> {
        let (len, buf) = u16::try_decode(buf)?;

        self.0 = String::from_utf8(buf[..len as usize].to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

impl<const N: u16> io::Write for FixedText<N> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.try_write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<const N: u16> io::Read for FixedText<N> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.try_read(buf)
    }
}
