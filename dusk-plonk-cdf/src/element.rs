mod fixed_text;
mod impls;
mod scalar;

use std::io;

use crate::{Config, Preamble};

pub use fixed_text::FixedText;
pub use scalar::Scalar;

/// Describe a CDF element
pub trait Element: Sized {
    /// Configuration for the serialization and deserialization
    type Config: for<'a> From<&'a Config>;

    /// A zeroed/default instance of the type.
    fn zeroed() -> Self;

    /// Serializable length
    ///
    /// Every element is a function of the config so seek/lookups will be constant-time.
    ///
    /// The serialized type must not contain more bytes than specified here. However, it might,
    /// optionally, use less bytes. Regardless, it will consume this defined amount of bytes during
    /// serialization.
    fn len(config: &Self::Config) -> usize;

    /// Write the type into the buffer.
    ///
    /// The buffer is guaranteed, provided a correct definition of [`Element::len`], to contain
    /// enough bytes to fully serialize the type - so length checks aren't required. Hence, this
    /// function is infallible.
    ///
    /// This will enforce the implementors to be designed to not hold stateful serialization,
    /// allowing greater flexibility of usage.
    fn to_buffer(&self, config: &Self::Config, buf: &mut [u8]);

    /// Deserialize the type from a given buffer
    ///
    /// As in [`Self::to_buffer`] the implementor of this function can assume the buffer is big
    /// enough to contain all the required bytes.
    fn try_from_buffer_in_place(&mut self, config: &Self::Config, buf: &[u8]) -> io::Result<()>;

    /// Perform the internal validations of the associated element
    fn validate(&self, preamble: &Preamble) -> io::Result<()>;

    /// Serialize the object into a bytes array.
    fn to_vec(&self, config: &Self::Config) -> Vec<u8> {
        let mut bytes = vec![0u8; Self::len(config)];

        self.to_buffer(config, &mut bytes);

        bytes
    }

    /// Create a new instance of the type from the provided buffer
    fn try_from_buffer(config: &Self::Config, buf: &[u8]) -> io::Result<Self> {
        let mut slf = Self::zeroed();

        slf.try_from_buffer_in_place(config, buf)?;

        Ok(slf)
    }

    /// Write an element from the buffer, and return the remainder bytes
    ///
    /// Assume its inside a validate buffer context
    fn try_decode_in_place<'a>(
        &mut self,
        config: &Self::Config,
        buf: &'a [u8],
    ) -> io::Result<&'a [u8]> {
        self.try_from_buffer_in_place(config, buf)
            .map(|_| &buf[Self::len(config)..])
    }

    /// Write an element from the buffer, and return the remainder bytes
    ///
    /// Assume its inside a validate buffer context
    fn try_decode<'a>(config: &Self::Config, buf: &'a [u8]) -> io::Result<(Self, &'a [u8])> {
        let mut slf = Self::zeroed();

        let buf = slf.try_decode_in_place(config, buf)?;

        Ok((slf, buf))
    }

    /// Read an element into the buffer, returning the remainder bytes
    ///
    /// Assume its inside a validate buffer context
    fn encode<'a>(&self, config: &Self::Config, buf: &'a mut [u8]) -> &'a mut [u8] {
        self.to_buffer(config, buf);

        &mut buf[Self::len(config)..]
    }

    /// Send the bytes representation of an element to a writer
    fn try_to_writer<W>(&self, mut writer: W, config: &Self::Config) -> io::Result<usize>
    where
        W: io::Write,
    {
        writer.write(&self.to_vec(config))
    }

    /// Fetch a new element from a reader
    fn try_from_reader<R>(mut reader: R, config: &Self::Config) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut slf = vec![0u8; Self::len(config)];
        let _ = reader.read(&mut slf)?;

        Self::try_from_buffer(config, &slf)
    }
}
