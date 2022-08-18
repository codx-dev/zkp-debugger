use std::marker::PhantomData;
use std::{io, mem};

use crate::{
    bytes, Config, DecodableElement, DecoderContext, Element, EncodableElement, EncoderContext,
    Preamble,
};

impl Element for bool {
    fn len(_ctx: &Config) -> usize {
        1
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

impl EncodableElement for bool {
    fn to_buffer(&self, _ctx: &mut EncoderContext, buf: &mut [u8]) {
        buf[0] = *self as u8;
    }
}

impl DecodableElement for bool {
    fn try_from_buffer_in_place<'b>(
        &mut self,
        ctx: &DecoderContext,
        buf: &'b [u8],
    ) -> io::Result<()> {
        Self::validate_buffer(ctx.config(), buf)?;

        *self = buf[0] != 0;

        Ok(())
    }
}

macro_rules! impl_num {
    ($t:ty) => {
        impl Element for $t {
            fn len(_ctx: &Config) -> usize {
                mem::size_of::<$t>()
            }

            fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
                Ok(())
            }
        }

        impl EncodableElement for $t {
            fn to_buffer(&self, _ctx: &mut EncoderContext, buf: &mut [u8]) {
                bytes::encode_bytes(&self.to_le_bytes(), buf);
            }
        }

        impl DecodableElement for $t {
            fn try_from_buffer_in_place<'b>(
                &mut self,
                ctx: &DecoderContext,
                buf: &'b [u8],
            ) -> io::Result<()> {
                Self::validate_buffer(ctx.config(), buf)?;

                const LEN: usize = mem::size_of::<$t>();

                let mut slf = [0u8; LEN];

                slf.copy_from_slice(&buf[..LEN]);

                *self = <$t>::from_le_bytes(slf);

                Ok(())
            }
        }
    };
}

impl_num!(u64);

// usize is implemented manually as u64 so the encoding will be platform agnostic
impl Element for usize {
    fn len(ctx: &Config) -> usize {
        u64::len(ctx)
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

impl EncodableElement for usize {
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]) {
        (*self as u64).to_buffer(ctx, buf)
    }
}

impl DecodableElement for usize {
    fn try_from_buffer_in_place<'b>(
        &mut self,
        ctx: &DecoderContext,
        buf: &'b [u8],
    ) -> io::Result<()> {
        let mut slf = 0u64;

        slf.try_from_buffer_in_place(ctx, buf)?;

        *self = slf as usize;

        Ok(())
    }
}

impl<T> Element for Option<T>
where
    T: Element,
{
    fn len(ctx: &Config) -> usize {
        T::len(ctx) + 1
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        match self {
            Some(t) => t.validate(preamble),
            None => Ok(()),
        }
    }
}

impl<T> EncodableElement for Option<T>
where
    T: EncodableElement,
{
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]) {
        let buf = self.is_some().encode(ctx, buf);

        // Will fill the space with zeroes, if `None`. This will guarantee deterministic
        // serialization, which will be desirable for checksum routines.
        match self {
            Some(t) => t.to_buffer(ctx, buf),
            None => buf[..Self::len(ctx.config()).saturating_sub(1)].fill(0),
        }
    }
}

impl<T> DecodableElement for Option<T>
where
    T: DecodableElement,
{
    fn try_from_buffer_in_place<'b>(
        &mut self,
        ctx: &DecoderContext,
        buf: &'b [u8],
    ) -> io::Result<()> {
        Self::validate_buffer(ctx.config(), buf)?;

        let (is_some, buf) = bool::try_decode(ctx, buf)?;
        match self {
            Some(t) if is_some => {
                t.try_from_buffer_in_place(ctx, buf)?;
            }

            None if is_some => {
                let t = T::try_from_buffer(ctx, buf)?;

                self.replace(t);
            }

            Some(_) | None => {
                self.take();
            }
        }

        Ok(())
    }
}

impl Element for () {
    fn len(_ctx: &Config) -> usize {
        0
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

impl EncodableElement for () {
    fn to_buffer(&self, _ctx: &mut EncoderContext, _buf: &mut [u8]) {}
}

impl DecodableElement for () {
    fn try_from_buffer_in_place<'b>(
        &mut self,
        _ctx: &DecoderContext,
        _buf: &'b [u8],
    ) -> io::Result<()> {
        Ok(())
    }
}

impl<T> Element for PhantomData<T> {
    fn len(_ctx: &Config) -> usize {
        0
    }

    fn validate(&self, _preamble: &Preamble) -> io::Result<()> {
        Ok(())
    }
}

impl<T> EncodableElement for PhantomData<T> {
    fn to_buffer(&self, _ctx: &mut EncoderContext, _buf: &mut [u8]) {}
}

impl<T> DecodableElement for PhantomData<T> {
    fn try_from_buffer_in_place<'b>(
        &mut self,
        _ctx: &DecoderContext,
        _buf: &'b [u8],
    ) -> io::Result<()> {
        Ok(())
    }
}
