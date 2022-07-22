use std::ops::Deref;
use std::{fmt, io, mem};

use crate::{bytes, AtomicConfig, Config, Context, ContextUnit, Element, Preamble};

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
        assert!(mem::size_of::<u16>() <= N as usize);

        s.truncate(N as usize - mem::size_of::<u16>());

        Self(s)
    }
}

impl<const N: u16> From<FixedText<N>> for String {
    fn from(t: FixedText<N>) -> Self {
        t.0
    }
}

impl<const N: u16> Element for FixedText<N> {
    type Config = Config;

    fn zeroed() -> Self {
        Self::default()
    }

    fn len(_config: &Self::Config) -> usize {
        N as usize
    }

    fn to_buffer(&self, _config: &Self::Config, context: &mut ContextUnit, buf: &mut [u8]) {
        let bytes = self.0.as_bytes();

        let buf = (bytes.len() as u16).encode(&AtomicConfig, context, buf);

        let _ = bytes::encode_bytes(bytes, buf);
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

        let (len, buf) = u16::try_decode(&AtomicConfig, context, buf)?;

        self.0 = String::from_utf8(buf[..len as usize].to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(())
    }

    fn validate(&self, _config: &Preamble) -> io::Result<()> {
        Ok(())
    }
}
