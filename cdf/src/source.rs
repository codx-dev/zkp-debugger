use std::path::PathBuf;
use std::{io, mem};

use crate::{
    Config, DecodableElement, DecoderContext, Element, EncodableElement, EncoderContext, Preamble,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct EncodedSource {
    line: u64,
    col: u64,
    contents_index: usize,
}

impl EncodedSource {
    const fn new(line: u64, col: u64, contents_index: usize) -> Self {
        Self {
            line,
            col,
            contents_index,
        }
    }
}

impl Element for EncodedSource {
    fn len(ctx: &Config) -> usize {
        2 * u64::len(ctx) + usize::len(ctx)
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.line.validate(preamble)?;
        self.col.validate(preamble)?;
        self.contents_index.validate(preamble)?;

        Ok(())
    }
}

impl EncodableElement for EncodedSource {
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]) {
        let buf = self.line.encode(ctx, buf);
        let buf = self.col.encode(ctx, buf);
        let _ = self.contents_index.encode(ctx, buf);
    }
}

impl DecodableElement for EncodedSource {
    fn try_from_buffer_in_place<'b>(
        &mut self,
        ctx: &DecoderContext,
        buf: &'b [u8],
    ) -> io::Result<()> {
        Self::validate_buffer(ctx.config(), buf)?;

        let buf = self.line.try_decode_in_place(&ctx, buf)?;
        let buf = self.col.try_decode_in_place(&ctx, buf)?;
        let _ = self.contents_index.try_decode_in_place(&ctx, buf)?;

        Ok(())
    }
}

/// Source file tripler that can be encoded into a CDF file
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EncodableSource {
    line: u64,
    col: u64,
    path: PathBuf,
}

impl EncodableSource {
    /// Create a new source instance
    pub const fn new(line: u64, col: u64, path: PathBuf) -> Self {
        Self { line, col, path }
    }
}

impl Element for EncodableSource {
    fn len(ctx: &Config) -> usize {
        EncodedSource::len(ctx)
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.line.validate(preamble)?;
        self.col.validate(preamble)?;

        Ok(())
    }
}

impl EncodableElement for EncodableSource {
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]) {
        let contents_index = ctx.add_path(self.path.clone());
        let encodable = EncodedSource::new(self.line, self.col, contents_index);

        encodable.to_buffer(ctx, buf)
    }
}

/// Source file decoded from a CDF file
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecodedSource<'a> {
    pub(crate) line: u64,
    pub(crate) col: u64,
    pub(crate) name: &'a str,
    pub(crate) contents: &'a str,
}

impl<'a> Element for DecodedSource<'a> {
    fn len(ctx: &Config) -> usize {
        EncodedSource::len(ctx)
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.line.validate(preamble)?;
        self.col.validate(preamble)?;

        Ok(())
    }
}

impl<'a> DecodableElement for DecodedSource<'a> {
    fn try_from_buffer_in_place<'x, 'b>(
        &'x mut self,
        ctx: &DecoderContext<'x>,
        buf: &'b [u8],
    ) -> io::Result<()> {
        let (encoded, _) = EncodedSource::try_decode(ctx, buf)?;
        let EncodedSource {
            line,
            col,
            contents_index,
        } = encoded;

        let name = ctx.fetch_name(contents_index).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "the source name wasn't available in the file cache",
            )
        })?;

        let contents = ctx.fetch_contents(contents_index).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "the source contents wasn't available in the file cache",
            )
        })?;

        self.line = line;
        self.col = col;

        // the compiler isn't smart enough here to understand that `self` is `'a`; hence the
        // context is also `'a`
        //
        // it is desirable to perform this safe change instead of taking every source as owned
        self.name = unsafe { mem::transmute::<&'x str, &'a str>(name) };
        self.contents = unsafe { mem::transmute::<&'x str, &'a str>(contents) };

        Ok(())
    }
}
