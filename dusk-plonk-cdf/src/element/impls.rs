use std::marker::PhantomData;
use std::{io, mem};

use crate::{bytes, AtomicConfig, Config, Context, ContextUnit, Element, Preamble};

impl Element for bool {
    type Config = AtomicConfig;

    fn zeroed() -> Self {
        false
    }

    fn len(_config: &Self::Config) -> usize {
        1
    }

    fn to_buffer(&self, _config: &Self::Config, _context: &mut ContextUnit, buf: &mut [u8]) {
        buf[0] = *self as u8;
    }

    fn try_from_buffer_in_place<S>(
        &mut self,
        _config: &Self::Config,
        _context: &mut Context<S>,
        buf: &[u8],
    ) -> io::Result<()> {
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

            fn to_buffer(
                &self,
                _config: &Self::Config,
                _context: &mut ContextUnit,
                buf: &mut [u8],
            ) {
                bytes::encode_bytes(&self.to_le_bytes(), buf);
            }

            fn try_from_buffer_in_place<S>(
                &mut self,
                _config: &Self::Config,
                _context: &mut Context<S>,
                buf: &[u8],
            ) -> io::Result<()> {
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

// usize is implemented manually as u64 so the encoding will be platform agnostic
impl Element for usize {
    type Config = AtomicConfig;

    fn zeroed() -> Self {
        0
    }

    fn len(_config: &Self::Config) -> usize {
        mem::size_of::<u64>()
    }

    fn to_buffer(&self, _config: &Self::Config, _context: &mut ContextUnit, buf: &mut [u8]) {
        bytes::encode_bytes(&self.to_le_bytes(), buf);
    }

    fn try_from_buffer_in_place<S>(
        &mut self,
        _config: &Self::Config,
        _context: &mut Context<S>,
        buf: &[u8],
    ) -> io::Result<()> {
        const LEN: usize = mem::size_of::<u64>();

        let mut slf = [0u8; LEN];

        slf.copy_from_slice(&buf[..LEN]);

        *self = u64::from_le_bytes(slf) as usize;

        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

impl<C, T> Element for Option<T>
where
    C: for<'a> From<&'a Config>,
    T: Element<Config = C>,
{
    type Config = C;

    fn zeroed() -> Self {
        None
    }

    fn len(config: &Self::Config) -> usize {
        T::len(config) + 1
    }

    fn to_buffer(&self, config: &Self::Config, context: &mut ContextUnit, buf: &mut [u8]) {
        let buf = self.is_some().encode(&AtomicConfig, context, buf);

        // Will fill the space with zeroes, if `None`. This will guarantee deterministic
        // serialization, which will be desirable for checksum routines.
        match self {
            Some(t) => t.to_buffer(config, context, buf),
            None => buf.fill(0),
        }
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
        let (is_some, buf) = bool::try_decode(&AtomicConfig, context, buf)?;

        match self {
            Some(t) if is_some => {
                t.try_from_buffer_in_place(config, context, buf)?;
            }

            None if is_some => {
                let t = T::try_from_buffer(config, context, buf)?;

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
    type Config = AtomicConfig;

    fn zeroed() -> Self {}

    fn len(_config: &Self::Config) -> usize {
        0
    }

    fn to_buffer(&self, _config: &Self::Config, _context: &mut ContextUnit, _buf: &mut [u8]) {}

    fn try_from_buffer_in_place<S>(
        &mut self,
        _config: &Self::Config,
        _context: &mut Context<S>,
        _buf: &[u8],
    ) -> io::Result<()> {
        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

impl<T> Element for PhantomData<T> {
    type Config = AtomicConfig;

    fn zeroed() -> Self {
        PhantomData
    }

    fn len(_config: &Self::Config) -> usize {
        0
    }

    fn to_buffer(&self, _config: &Self::Config, _context: &mut ContextUnit, _buf: &mut [u8]) {}

    fn try_from_buffer_in_place<S>(
        &mut self,
        _config: &Self::Config,
        _context: &mut Context<S>,
        _buf: &[u8],
    ) -> io::Result<()> {
        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn validate_option_works() {
    Some(34873u64)
        .validate(&Default::default())
        .expect("default config validate should pass");

    let opt: Option<u64> = None;

    opt.validate(&Default::default())
        .expect("default config validate should pass");
}

#[test]
fn validate_unit_works() {
    ().validate(&Default::default())
        .expect("default config validate should pass");
}
