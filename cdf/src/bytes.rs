/// Read an element into the buffer, returning the remainder bytes
///
/// Assume its inside a validate buffer context
pub fn encode_bytes<'a>(source: &[u8], buf: &'a mut [u8]) -> &'a mut [u8] {
    buf[..source.len()].copy_from_slice(source);

    &mut buf[source.len()..]
}
