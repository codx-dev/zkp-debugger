use std::marker::PhantomData;
use std::{io, mem};

use crate::{bytes, Config, Element, Preamble, AtomicConfig};

impl Element for bool {
    type Config = AtomicConfig;

    fn zeroed() -> Self {
        false
    }

    fn len(_config: &Self::Config) -> usize {
        1
    }

    fn to_buffer(&self, _config: &Self::Config, buf: &mut [u8]) {
        buf[0] = *self as u8;
    }

    fn try_from_buffer_in_place(&mut self, _config: &Self::Config, buf: &[u8]) -> io::Result<()> {
        *self = buf[0] != 0;

        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

macro_rules! impl_num {
    ($t:ty) => {
        impl Element for $t {
            type Config = AtomicConfig;

            fn zeroed() -> Self {
                0
            }

            fn len(_config: &Self::Config) -> usize {
                mem::size_of::<$t>()
            }

            fn to_buffer(&self, _config: &Self::Config, buf: &mut [u8]) {
                bytes::encode_bytes(&self.to_le_bytes(), buf);
            }

            fn try_from_buffer_in_place(&mut self, _config: &Self::Config, buf: &[u8]) -> io::Result<()> {
                const LEN: usize = mem::size_of::<$t>();

                let mut slf = [0u8; LEN];

                slf.copy_from_slice(&buf[..LEN]);

                *self = <$t>::from_le_bytes(slf);

                Ok(())
            }

            fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
                Ok(())
            }
        }
    };
}

impl_num!(u16);
impl_num!(u64);
impl_num!(usize);

impl<T> Element for Option<T>
where
    T: Element<Config = AtomicConfig>,
{
    type Config = AtomicConfig;

    fn zeroed() -> Self {
        None
    }

    fn len(config: &Self::Config) -> usize {
        T::len(config) + 1
    }

    fn to_buffer(&self, config: &Self::Config, buf: &mut [u8]) {
        let buf = self.is_some().encode(&AtomicConfig, buf);

        // Will fill the space with zeroes, if `None`. This will guarantee deterministic
        // serialization, which will be desirable for checksum routines.
        match self {
            Some(t) => t.to_buffer(config, buf),
            None => buf.fill(0),
        }
    }

    fn try_from_buffer_in_place(&mut self, config: &Self::Config, buf: &[u8]) -> io::Result<()> {
        let (is_some, buf) = bool::try_decode(&AtomicConfig, buf)?;

        match self {
            Some(t) if is_some => {
                t.try_from_buffer_in_place(config, buf)?;
            }

            None if is_some => {
                let t = T::try_from_buffer(config, buf)?;

                self.replace(t);
            }

            Some(_) | None => {
                self.take();
            }
        }

        Ok(())
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        match self {
            Some(t) => t.validate(preamble),
            None => Ok(()),
        }
    }
}

impl Element for () {
    type Config = Config;

    fn zeroed() -> Self {
        ()
    }

    fn len(_config: &Self::Config) -> usize {
        0
    }

    fn to_buffer(&self, _config: &Self::Config, _buf: &mut [u8]) {}

    fn try_from_buffer_in_place(&mut self, _config: &Self::Config, _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

impl<T> Element for PhantomData<T> {
    type Config = Config;

    fn zeroed() -> Self {
        PhantomData
    }

    fn len(_config: &Self::Config) -> usize {
        0
    }

    fn to_buffer(&self, _config: &Self::Config, _buf: &mut [u8]) {}

    fn try_from_buffer_in_place(&mut self, _config: &Self::Config, _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}
