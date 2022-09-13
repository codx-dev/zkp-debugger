use std::io;

use crate::{
    Config, DecodableElement, DecodedSource, DecoderContext, Element, EncodableElement,
    EncodableSource, EncoderContext, Polynomial, Preamble,
};

/// Constraint representation that can be encoded into a CDF file
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EncodableConstraint {
    id: usize,
    polynomial: Polynomial,
    source: EncodableSource,
}

impl EncodableConstraint {
    /// Create a new encodable constraint instance
    pub const fn new(id: usize, polynomial: Polynomial, source: EncodableSource) -> Self {
        Self {
            id,
            polynomial,
            source,
        }
    }

    /// Id of the constraint in the constraint system
    pub const fn id(&self) -> usize {
        self.id
    }

    /// Polynomial representation
    pub const fn polynomial(&self) -> &Polynomial {
        &self.polynomial
    }

    /// Source reference to be encoded
    pub const fn source(&self) -> &EncodableSource {
        &self.source
    }
}

impl Element for EncodableConstraint {
    fn len(ctx: &Config) -> usize {
        usize::len(ctx) + Polynomial::len(ctx) + EncodableSource::len(ctx)
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.id.validate(preamble)?;
        self.polynomial.validate(preamble)?;
        self.source.validate(preamble)?;

        Ok(())
    }
}

impl EncodableElement for EncodableConstraint {
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]) {
        let buf = self.id.encode(ctx, buf);
        let buf = self.polynomial.encode(ctx, buf);
        let _ = self.source.encode(ctx, buf);
    }
}

/// Decoded constraint from a CDF file
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Constraint<'a> {
    id: usize,
    polynomial: Polynomial,
    source: DecodedSource<'a>,
}

impl<'a> Constraint<'a> {
    /// Constructor private to the crate because witness is suposed to be created from the cdf
    /// file
    pub(crate) const fn _new(id: usize, polynomial: Polynomial, source: DecodedSource<'a>) -> Self {
        Self {
            id,
            polynomial,
            source,
        }
    }

    /// Id of the constraint in the constraint system
    pub const fn id(&self) -> usize {
        self.id
    }

    /// Polynomial representation
    pub const fn polynomial(&self) -> &Polynomial {
        &self.polynomial
    }

    /// Line of the source code
    pub const fn line(&self) -> u64 {
        self.source.line
    }

    /// Column of the source code
    pub const fn col(&self) -> u64 {
        self.source.col
    }

    /// Source file name
    pub const fn name(&self) -> &str {
        self.source.name
    }

    /// Source code contents
    pub const fn contents(&self) -> &str {
        self.source.contents
    }
}

impl<'a> Element for Constraint<'a> {
    fn len(ctx: &Config) -> usize {
        usize::len(ctx) + Polynomial::len(ctx) + DecodedSource::len(ctx)
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.id.validate(preamble)?;
        self.polynomial.validate(preamble)?;
        self.source.validate(preamble)?;

        Ok(())
    }
}

impl<'a> DecodableElement for Constraint<'a> {
    fn try_from_buffer_in_place<'x, 'b>(
        &'x mut self,
        ctx: &DecoderContext<'x>,
        buf: &'b [u8],
    ) -> io::Result<()> {
        Self::validate_buffer(ctx.config(), buf)?;

        let buf = self.id.try_decode_in_place(ctx, buf)?;
        let buf = self.polynomial.try_decode_in_place(ctx, buf)?;
        let _ = self.source.try_decode_in_place(ctx, buf)?;

        Ok(())
    }
}
