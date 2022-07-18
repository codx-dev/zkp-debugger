use core::borrow::Borrow;
use std::io;

use crate::{Element, Preamble};

/// Read an element into the buffer, returning the remainder bytes
///
/// Assume its inside a validate buffer context
pub fn encode_bytes<'a>(source: &[u8], buf: &'a mut [u8]) -> &'a mut [u8] {
    buf[..source.len()].copy_from_slice(source);

    &mut buf[source.len()..]
}

/// Send the bytes representation of an element to a writer
pub fn try_to_writer<W, B, L>(mut writer: W, preamble: &Preamble, e: B) -> io::Result<usize>
where
    W: io::Write,
    B: Borrow<L>,
    L: Element,
{
    writer.write(&e.borrow().to_vec(preamble))
}

/// Fetch a new element from a reader
pub fn try_from_reader<R, L>(mut reader: R, preamble: &Preamble) -> io::Result<L>
where
    R: io::Read,
    L: Element,
{
    let mut slf = vec![0u8; L::len(preamble)];
    let _ = reader.read(&mut slf)?;

    L::try_from_buffer(preamble, &slf)
}
