use std::{io, mem};

use crate::{
    Config, Constraint, DecodableElement, DecoderContext, Element, EncodableElement,
    EncoderContext, Witness,
};

/// Metadata information of the CDF file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preamble {
    /// Witnesses count
    pub witnesses: usize,
    /// Constraints count
    pub constraints: usize,
    /// Configuration parameters for encoding and decoding
    pub config: Config,
}

impl Preamble {
    /// Serialized length
    pub const LEN: usize = 2 * mem::size_of::<usize>() + Config::LEN;

    /// Create a new preamble instance
    pub const fn new(witnesses: usize, constraints: usize, config: Config) -> Self {
        Self {
            witnesses,
            constraints,
            config,
        }
    }

    /// Witness offset in CDF, from an index
    pub fn witness_offset(&self, idx: usize) -> Option<usize> {
        (idx < self.witnesses).then(|| Self::LEN + idx * Witness::len(&self.config))
    }

    /// Constraint offset in CDF, from an index
    pub fn constraint_offset(&self, idx: usize) -> Option<usize> {
        (idx < self.constraints).then(|| {
            Self::LEN
                + self.witnesses * Witness::len(&self.config)
                + idx * Constraint::len(&self.config)
        })
    }

    /// Cache starting position
    pub fn source_cache_offset(&self) -> usize {
        Self::LEN
            + self.witnesses * Witness::len(&self.config)
            + self.constraints * Constraint::len(&self.config)
    }
}

impl Default for Preamble {
    fn default() -> Self {
        Self {
            witnesses: 1,
            constraints: 0,
            config: Default::default(),
        }
    }
}

impl Element for Preamble {
    fn len(ctx: &Config) -> usize {
        2 * usize::len(ctx) + Config::len(ctx)
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.witnesses.validate(preamble)?;
        self.constraints.validate(preamble)?;
        self.config.validate(preamble)?;

        Ok(())
    }
}

impl EncodableElement for Preamble {
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]) {
        let buf = self.witnesses.encode(ctx, buf);
        let buf = self.constraints.encode(ctx, buf);
        let _ = self.config.encode(ctx, buf);
    }
}

impl DecodableElement for Preamble {
    fn try_from_buffer_in_place<'a, 'b>(
        &'a mut self,
        ctx: &DecoderContext<'a>,
        buf: &'b [u8],
    ) -> io::Result<()> {
        Self::validate_buffer(ctx.config(), buf)?;

        let buf = self.witnesses.try_decode_in_place(&ctx, buf)?;
        let buf = self.constraints.try_decode_in_place(&ctx, buf)?;
        let _ = self.config.try_decode_in_place(&ctx, buf)?;

        Ok(())
    }
}
