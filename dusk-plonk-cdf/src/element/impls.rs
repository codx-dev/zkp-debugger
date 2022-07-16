use std::marker::PhantomData;
use std::{io, mem};

use crate::Preamble;

use super::Element;

impl Element for bool {
    const LEN: usize = 1;

    fn zeroed() -> Self {
        false
    }

    fn to_buffer(&self, buf: &mut [u8]) {
        buf[0] = *self as u8;
    }

    fn try_from_buffer_in_place(&mut self, buf: &[u8]) -> io::Result<()> {
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
            const LEN: usize = mem::size_of::<$t>();

            fn zeroed() -> Self {
                0
            }

            fn to_buffer(&self, buf: &mut [u8]) {
                Self::encode_bytes(&self.to_le_bytes(), buf);
            }

            fn try_from_buffer_in_place(&mut self, buf: &[u8]) -> io::Result<()> {
                let mut slf = [0u8; Self::LEN];

                slf.copy_from_slice(&buf[..Self::LEN]);

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
    const LEN: usize = T::LEN + 1;

    fn zeroed() -> Self {
        None
    }

    fn to_buffer(&self, buf: &mut [u8]) {
        let buf = self.is_some().encode(buf);

        // Will fill the space with zeroes, if `None`. This will guarantee deterministic
        // serialization, which will be desirable for checksum routines.
        match self {
            Some(t) => t.to_buffer(buf),
            None => buf.fill(0),
        }
    }

    fn try_from_buffer_in_place(&mut self, buf: &[u8]) -> io::Result<()> {
        let (is_some, buf) = bool::try_decode(buf)?;

        match self {
            Some(t) if is_some => {
                t.try_from_buffer_in_place(buf)?;
            }

            None if is_some => {
                let t = T::try_from_buffer(buf)?;

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
    const LEN: usize = 0;

    fn zeroed() -> Self {
        ()
    }

    fn to_buffer(&self, _buf: &mut [u8]) {}

    fn try_from_buffer_in_place(&mut self, _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

impl<T> Element for PhantomData<T> {
    const LEN: usize = 0;

    fn zeroed() -> Self {
        PhantomData
    }

    fn to_buffer(&self, _buf: &mut [u8]) {}

    fn try_from_buffer_in_place(&mut self, _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}
