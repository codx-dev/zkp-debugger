mod fixed_text;
mod impls;
mod scalar;

use std::io;

use super::Preamble;

pub use fixed_text::FixedText;
pub use scalar::Scalar;

/// Describe a CDF element
pub trait Element: Sized {
    /// Serializable length
    ///
    /// Every element is of constant size so seek/lookups in its binary representation is
    /// constant-time and trivial.
    ///
    /// The serialized type must not contain more bytes than specified here. However, it might,
    /// optionally, use less bytes. Regardless, it will consume this defined amount of bytes during
    /// serialization.
    const LEN: usize;

    /// A zeroed/default instance of the type.
    fn zeroed() -> Self;

    /// Write the type into the buffer.
    ///
    /// The buffer is guaranteed, provided a correct definition of [`Element::LEN`], to contain
    /// enough bytes to fully serialize the type - so length checks aren't required. Hence, this
    /// function is infallible.
    ///
    /// This will enforce the implementors to be designed to not hold stateful serialization,
    /// allowing greater flexibility of usage.
    fn to_buffer(&self, buf: &mut [u8]);

    /// Deserialize the type from a given buffer
    ///
    /// As in [`Self::to_buffer`] the implementor of this function can assume the buffer is big
    /// enough to contain all the required bytes.
    fn try_from_buffer_in_place(&mut self, buf: &[u8]) -> io::Result<()>;

    /// Perform the internal validations of the associated element
    fn validate(&self, preamble: &Preamble) -> io::Result<()>;

    /// Serialize the object into a bytes array.
    fn to_vec(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; Self::LEN];

        self.to_buffer(&mut bytes);

        bytes
    }

    /// Create a new instance of the type from the provided buffer
    fn try_from_buffer(buf: &[u8]) -> io::Result<Self> {
        let mut slf = Self::zeroed();

        slf.try_from_buffer_in_place(buf)?;

        Ok(slf)
    }

    /// [`io::Read::read`] convenience implementation, without requiring interior mutability
    fn try_read(&self, buf: &mut [u8]) -> io::Result<usize> {
        Self::validate_buffer(buf)?;

        self.to_buffer(buf);

        Ok(Self::LEN)
    }

    /// [`io::Write::write`] convenience implementation
    fn try_write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Self::validate_buffer(buf)?;

        self.try_from_buffer_in_place(buf)?;

        Ok(Self::LEN)
    }

    /// Assert the given buffer is big enough to store the element
    fn validate_buffer(buf: &[u8]) -> io::Result<()> {
        if buf.len() < Self::LEN {
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
    fn try_decode_in_place<'a>(&mut self, buf: &'a [u8]) -> io::Result<&'a [u8]> {
        self.try_from_buffer_in_place(buf)
            .map(|_| &buf[Self::LEN..])
    }

    /// Write an element from the buffer, and return the remainder bytes
    ///
    /// Assume its inside a validate buffer context
    fn try_decode(buf: &[u8]) -> io::Result<(Self, &[u8])> {
        let mut slf = Self::zeroed();

        let buf = slf.try_decode_in_place(buf)?;

        Ok((slf, buf))
    }

    /// Read an element into the buffer, returning the remainder bytes
    ///
    /// Assume its inside a validate buffer context
    fn encode<'a>(&self, buf: &'a mut [u8]) -> &'a mut [u8] {
        self.to_buffer(buf);

        &mut buf[Self::LEN..]
    }

    /// Read an element into the buffer, returning the remainder bytes
    ///
    /// Assume its inside a validate buffer context
    fn encode_bytes<'a>(source: &[u8], buf: &'a mut [u8]) -> &'a mut [u8] {
        buf[..source.len()].copy_from_slice(source);

        &mut buf[source.len()..]
    }

    /// Send the element to a writer implementation
    fn to_writer<W>(&self, mut writer: W) -> io::Result<usize>
    where
        W: io::Write,
    {
        writer.write(&self.to_vec())
    }

    /// Fetch a new element from a reader
    fn from_reader<R>(mut reader: R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut slf = vec![0u8; Self::LEN];
        let _ = reader.read(&mut slf)?;

        Self::try_from_buffer(&slf)
    }
}
