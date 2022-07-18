use std::marker::PhantomData;
use std::{io, mem};

use crate::{bytes, Preamble};

use super::Element;

impl Element for bool {
    fn zeroed() -> Self {
        false
    }

    fn len(_preamble: &Preamble) -> usize {
        1
    }

    fn to_buffer(&self, _preamble: &Preamble, buf: &mut [u8]) {
        buf[0] = *self as u8;
    }

    fn try_from_buffer_in_place(&mut self, _preamble: &Preamble, buf: &[u8]) -> io::Result<()> {
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
            fn zeroed() -> Self {
                0
            }

            fn len(_preamble: &Preamble) -> usize {
                mem::size_of::<$t>()
            }

            fn to_buffer(&self, _preamble: &Preamble, buf: &mut [u8]) {
                bytes::encode_bytes(&self.to_le_bytes(), buf);
            }

            fn try_from_buffer_in_place(
                &mut self,
                _preamble: &Preamble,
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
impl_num!(usize);

impl<T> Element for Option<T>
where
    T: Element,
{
    fn zeroed() -> Self {
        None
    }

    fn len(preamble: &Preamble) -> usize {
        T::len(preamble) + 1
    }

    fn to_buffer(&self, preamble: &Preamble, buf: &mut [u8]) {
        let buf = self.is_some().encode(preamble, buf);

        // Will fill the space with zeroes, if `None`. This will guarantee deterministic
        // serialization, which will be desirable for checksum routines.
        match self {
            Some(t) => t.to_buffer(preamble, buf),
            None => buf.fill(0),
        }
    }

    fn try_from_buffer_in_place(&mut self, preamble: &Preamble, buf: &[u8]) -> io::Result<()> {
        let (is_some, buf) = bool::try_decode(preamble, buf)?;

        match self {
            Some(t) if is_some => {
                t.try_from_buffer_in_place(preamble, buf)?;
            }

            None if is_some => {
                let t = T::try_from_buffer(preamble, buf)?;

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
    fn zeroed() -> Self {
        ()
    }

    fn len(_preamble: &Preamble) -> usize {
        0
    }

    fn to_buffer(&self, _preamble: &Preamble, _buf: &mut [u8]) {}

    fn try_from_buffer_in_place(&mut self, _preamble: &Preamble, _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

impl<T> Element for PhantomData<T> {
    fn zeroed() -> Self {
        PhantomData
    }

    fn len(_preamble: &Preamble) -> usize {
        0
    }

    fn to_buffer(&self, _preamble: &Preamble, _buf: &mut [u8]) {}

    fn try_from_buffer_in_place(&mut self, _preamble: &Preamble, _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}
