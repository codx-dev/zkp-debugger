mod impls;
mod scalar;

use std::io;

use crate::{Config, DecoderContext, EncoderContext, Preamble};

pub use scalar::Scalar;

/// Element that can be encoded into a CDF file
pub trait EncodableElement: Element {
    /// Write the type into the buffer.
    ///
    /// # Panics
    ///
    /// The buffer must, provided a correct definition of [`Element::len`], contain enough bytes to
    /// fully serialize the type. This can be checkedvia [`Element::validate_buffer_len`].
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]);

    /// Serialize the object into a bytes array.
    fn to_vec(&self, ctx: &mut EncoderContext) -> Vec<u8> {
        let len = Self::len(ctx.config());
        let mut bytes = vec![0u8; len];

        self.to_buffer(ctx, &mut bytes);

        bytes
    }

    /// Read an element into the buffer, returning the remainder bytes
    ///
    /// Assume its inside a validate buffer context
    fn encode<'a>(&self, ctx: &mut EncoderContext, buf: &'a mut [u8]) -> &'a mut [u8] {
        self.to_buffer(ctx, buf);

        &mut buf[Self::len(ctx.config())..]
    }

    /// Send the bytes representation of an element to a writer
    fn try_to_writer<W>(&self, mut writer: W, ctx: &mut EncoderContext) -> io::Result<usize>
    where
        W: io::Write,
    {
        writer.write(&self.to_vec(ctx))
    }
}

/// Element that can be decoded from a CDF file
pub trait DecodableElement: Sized + Element {
    /// Deserialize the type from a given buffer
    ///
    /// As in [`EncodableElement::to_buffer`] the implementor of this function can assume the buffer is big
    /// enough to contain all the required bytes.
    fn try_from_buffer_in_place<'a, 'b>(
        &'a mut self,
        ctx: &DecoderContext<'a>,
        buf: &'b [u8],
    ) -> io::Result<()>;

    /// Create a new instance of the type from the provided buffer
    fn try_from_buffer<'b>(ctx: &DecoderContext, buf: &'b [u8]) -> io::Result<Self> {
        let mut slf = Self::default();

        slf.try_from_buffer_in_place(ctx, buf)?;

        Ok(slf)
    }

    /// Write an element from the buffer, and return the remainder bytes
    ///
    /// Assume its inside a validate buffer context
    fn try_decode_in_place<'a, 'b>(
        &mut self,
        ctx: &DecoderContext<'a>,
        buf: &'b [u8],
    ) -> io::Result<&'b [u8]> {
        self.try_from_buffer_in_place(ctx, buf)
            .map(|_| &buf[Self::len(ctx.config())..])
    }

    /// Write an element from the buffer, and return the remainder bytes
    ///
    /// Assume its inside a validate buffer context
    fn try_decode<'a, 'b>(ctx: &DecoderContext<'a>, buf: &'b [u8]) -> io::Result<(Self, &'b [u8])> {
        let mut slf = Self::default();

        let buf = slf.try_decode_in_place(ctx, buf)?;

        Ok((slf, buf))
    }

    /// Fetch a new element from a context
    fn try_from_reader<R>(ctx: &DecoderContext, mut reader: R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut slf = vec![0u8; Self::len(ctx.config())];

        let _ = reader.read(&mut slf)?;

        Self::try_from_buffer(ctx, &slf)
    }
}

/// Describe a CDF element
pub trait Element: Default {
    /// Serializable length
    ///
    /// Every element is a function of the config so seek/lookups will be constant-time.
    ///
    /// The serialized type must not contain more bytes than specified here. However, it might,
    /// optionally, use less bytes. Regardless, it will consume this defined amount of bytes during
    /// serialization.
    fn len(ctx: &Config) -> usize;

    /// Perform the internal validations of the associated element
    fn validate(&self, preamble: &Preamble) -> io::Result<()>;

    /// Assert the buffer is big enough to store the type
    fn validate_buffer(config: &Config, buffer: &[u8]) -> io::Result<()> {
        if buffer.len() < Self::len(config) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "the provided buffer isn't big enough",
            ));
        }

        Ok(())
    }
}
