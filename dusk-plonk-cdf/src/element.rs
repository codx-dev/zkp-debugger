mod fixed_text;
mod impls;
mod scalar;

use std::io;

use super::Preamble;

pub use fixed_text::FixedText;
pub use scalar::Scalar;

/// Describe a CDF element
pub trait Element: Sized {
    /// A zeroed/default instance of the type.
    fn zeroed() -> Self;

    /// Serializable length
    ///
    /// Every element is a function of the preamble so seek/lookups will be constant-time.
    ///
    /// The serialized type must not contain more bytes than specified here. However, it might,
    /// optionally, use less bytes. Regardless, it will consume this defined amount of bytes during
    /// serialization.
    fn len(preamble: &Preamble) -> usize;

    /// Write the type into the buffer.
    ///
    /// The buffer is guaranteed, provided a correct definition of [`Element::len`], to contain
    /// enough bytes to fully serialize the type - so length checks aren't required. Hence, this
    /// function is infallible.
    ///
    /// This will enforce the implementors to be designed to not hold stateful serialization,
    /// allowing greater flexibility of usage.
    fn to_buffer(&self, preamble: &Preamble, buf: &mut [u8]);

    /// Deserialize the type from a given buffer
    ///
    /// As in [`Self::to_buffer`] the implementor of this function can assume the buffer is big
    /// enough to contain all the required bytes.
    fn try_from_buffer_in_place(&mut self, preamble: &Preamble, buf: &[u8]) -> io::Result<()>;

    /// Perform the internal validations of the associated element
    fn validate(&self, preamble: &Preamble) -> io::Result<()>;

    /// Serialize the object into a bytes array.
    fn to_vec(&self, preamble: &Preamble) -> Vec<u8> {
        let mut bytes = vec![0u8; Self::len(preamble)];

        self.to_buffer(preamble, &mut bytes);

        bytes
    }

    /// Create a new instance of the type from the provided buffer
    fn try_from_buffer(preamble: &Preamble, buf: &[u8]) -> io::Result<Self> {
        let mut slf = Self::zeroed();

        slf.try_from_buffer_in_place(preamble, buf)?;

        Ok(slf)
    }

    /// Assert the given buffer is big enough to store the element
    fn validate_buffer(preamble: &Preamble, buf: &[u8]) -> io::Result<()> {
        if buf.len() < Self::len(preamble) {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "the provided buffer isn't big enough to store a circuit definition format element",
            ));
        }

        Ok(())
    }

    /// Write an element from the buffer, and return the remainder bytes
    ///
    /// Assume its inside a validate buffer context
    fn try_decode_in_place<'a>(
        &mut self,
        preamble: &Preamble,
        buf: &'a [u8],
    ) -> io::Result<&'a [u8]> {
        self.try_from_buffer_in_place(preamble, buf)
            .map(|_| &buf[Self::len(preamble)..])
    }

    /// Write an element from the buffer, and return the remainder bytes
    ///
    /// Assume its inside a validate buffer context
    fn try_decode<'a>(preamble: &Preamble, buf: &'a [u8]) -> io::Result<(Self, &'a [u8])> {
        let mut slf = Self::zeroed();

        let buf = slf.try_decode_in_place(preamble, buf)?;

        Ok((slf, buf))
    }

    /// Read an element into the buffer, returning the remainder bytes
    ///
    /// Assume its inside a validate buffer context
    fn encode<'a>(&self, preamble: &Preamble, buf: &'a mut [u8]) -> &'a mut [u8] {
        self.to_buffer(preamble, buf);

        &mut buf[Self::len(preamble)..]
    }
}
