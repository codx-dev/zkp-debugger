use std::io;

use crate::{
    Config, DecodableElement, DecodedSource, DecoderContext, Element, EncodableElement,
    EncodableSource, EncoderContext, Preamble, Scalar,
};

/// Witness that can be encoded into a CDF file
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EncodableWitness {
    id: usize,
    constraint: Option<usize>,
    value: Scalar,
    source: EncodableSource,
}

impl EncodableWitness {
    /// Create a new encodable witness instance
    pub const fn new(
        id: usize,
        constraint: Option<usize>,
        value: Scalar,
        source: EncodableSource,
    ) -> Self {
        Self {
            id,
            constraint,
            value,
            source,
        }
    }
}

impl Element for EncodableWitness {
    fn len(ctx: &Config) -> usize {
        usize::len(ctx) + <Option<usize>>::len(ctx) + Scalar::len(ctx) + EncodableSource::len(ctx)
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.id.validate(preamble)?;
        self.constraint.validate(preamble)?;
        self.value.validate(preamble)?;
        self.source.validate(preamble)?;

        Ok(())
    }
}

impl EncodableElement for EncodableWitness {
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]) {
        let buf = self.id.encode(ctx, buf);
        let buf = self.constraint.encode(ctx, buf);
        let buf = self.value.encode(ctx, buf);
        let _ = self.source.encode(ctx, buf);
    }
}

/// Witness decoded from a CDF file
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Witness<'a> {
    id: usize,
    constraint: Option<usize>,
    value: Scalar,
    source: DecodedSource<'a>,
}

impl<'a> Witness<'a> {
    /// Id of the witness in the constraint system
    pub const fn id(&self) -> usize {
        self.id
    }

    /// Constraint that originated the witness
    pub const fn constraint(&self) -> Option<usize> {
        self.constraint
    }

    /// Value of the witness in the constraint system
    pub const fn value(&self) -> &Scalar {
        &self.value
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

impl<'a> Element for Witness<'a> {
    fn len(ctx: &Config) -> usize {
        usize::len(ctx) + <Option<usize>>::len(ctx) + Scalar::len(ctx) + DecodedSource::len(ctx)
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.id.validate(preamble)?;
        self.constraint.validate(preamble)?;
        self.value.validate(preamble)?;
        self.source.validate(preamble)?;

        Ok(())
    }
}

impl<'a> DecodableElement for Witness<'a> {
    fn try_from_buffer_in_place<'x, 'b>(
        &'x mut self,
        ctx: &DecoderContext<'x>,
        buf: &'b [u8],
    ) -> io::Result<()> {
        Self::validate_buffer(ctx.config(), buf)?;

        let buf = self.id.try_decode_in_place(&ctx, buf)?;
        let buf = self.constraint.try_decode_in_place(&ctx, buf)?;
        let buf = self.value.try_decode_in_place(&ctx, buf)?;
        let _ = self.source.try_decode_in_place(&ctx, buf)?;

        Ok(())
    }
}
