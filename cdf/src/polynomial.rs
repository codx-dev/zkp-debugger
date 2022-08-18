use std::io;

use crate::{
    Config, DecodableElement, DecoderContext, Element, EncodableElement, EncoderContext, Preamble,
    Scalar,
};

/// Polynomial selectors
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selectors {
    /// Qm (mult) selector
    pub qm: Scalar,
    /// Ql (left) selector
    pub ql: Scalar,
    /// Qr (right) selector
    pub qr: Scalar,
    /// Qd (fourth) selector
    pub qd: Scalar,
    /// Qc (constant) selector
    pub qc: Scalar,
    /// Qo (output) selector
    pub qo: Scalar,
    /// Public input
    pub pi: Scalar,
    /// Qarith (arithmetic) internal selector
    pub qarith: Scalar,
    /// Qlogic (logical) internal selector
    pub qlogic: Scalar,
    /// Qrange (range check) internal selector
    pub qrange: Scalar,
    /// Qgroup_variable (ecc group variable add) internal selector
    pub qgroup_variable: Scalar,
    /// Qgroup_fixed (ecc group fixed add) internal selector
    pub qfixed_add: Scalar,
}

impl Element for Selectors {
    fn len(ctx: &Config) -> usize {
        12 * Scalar::len(ctx)
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.qm.validate(preamble)?;
        self.ql.validate(preamble)?;
        self.qr.validate(preamble)?;
        self.qd.validate(preamble)?;
        self.qc.validate(preamble)?;
        self.qo.validate(preamble)?;
        self.pi.validate(preamble)?;
        self.qarith.validate(preamble)?;
        self.qlogic.validate(preamble)?;
        self.qrange.validate(preamble)?;
        self.qgroup_variable.validate(preamble)?;
        self.qfixed_add.validate(preamble)?;

        Ok(())
    }
}

impl EncodableElement for Selectors {
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]) {
        let buf = self.qm.encode(ctx, buf);
        let buf = self.ql.encode(ctx, buf);
        let buf = self.qr.encode(ctx, buf);
        let buf = self.qd.encode(ctx, buf);
        let buf = self.qc.encode(ctx, buf);
        let buf = self.qo.encode(ctx, buf);
        let buf = self.pi.encode(ctx, buf);
        let buf = self.qarith.encode(ctx, buf);
        let buf = self.qlogic.encode(ctx, buf);
        let buf = self.qrange.encode(ctx, buf);
        let buf = self.qgroup_variable.encode(ctx, buf);
        let _ = self.qfixed_add.encode(ctx, buf);
    }
}

impl DecodableElement for Selectors {
    fn try_from_buffer_in_place<'a, 'b>(
        &'a mut self,
        ctx: &DecoderContext<'a>,
        buf: &'b [u8],
    ) -> io::Result<()> {
        Self::validate_buffer(ctx.config(), buf)?;

        let buf = self.qm.try_decode_in_place(ctx, buf)?;
        let buf = self.ql.try_decode_in_place(ctx, buf)?;
        let buf = self.qr.try_decode_in_place(ctx, buf)?;
        let buf = self.qd.try_decode_in_place(ctx, buf)?;
        let buf = self.qc.try_decode_in_place(ctx, buf)?;
        let buf = self.qo.try_decode_in_place(ctx, buf)?;
        let buf = self.pi.try_decode_in_place(ctx, buf)?;
        let buf = self.qarith.try_decode_in_place(ctx, buf)?;
        let buf = self.qlogic.try_decode_in_place(ctx, buf)?;
        let buf = self.qrange.try_decode_in_place(ctx, buf)?;
        let buf = self.qgroup_variable.try_decode_in_place(ctx, buf)?;
        let _ = self.qfixed_add.try_decode_in_place(ctx, buf)?;

        Ok(())
    }
}

/// Polynomial witnesses allocated to a constraint system
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WiredWitnesses {
    /// Wired `a`
    pub a: usize,
    /// Wired `b`
    pub b: usize,
    /// Wired `d` (fourth)
    pub d: usize,
    /// Wired `o` (output)
    pub o: usize,
}

impl Element for WiredWitnesses {
    fn len(ctx: &Config) -> usize {
        4 * usize::len(ctx)
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.a.validate(preamble)?;
        self.b.validate(preamble)?;
        self.d.validate(preamble)?;
        self.o.validate(preamble)?;

        Ok(())
    }
}

impl EncodableElement for WiredWitnesses {
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]) {
        let buf = self.a.encode(ctx, buf);
        let buf = self.b.encode(ctx, buf);
        let buf = self.d.encode(ctx, buf);
        let _ = self.o.encode(ctx, buf);
    }
}

impl DecodableElement for WiredWitnesses {
    fn try_from_buffer_in_place<'a, 'b>(
        &'a mut self,
        ctx: &DecoderContext<'a>,
        buf: &'b [u8],
    ) -> io::Result<()> {
        let buf = self.a.try_decode_in_place(ctx, buf)?;
        let buf = self.b.try_decode_in_place(ctx, buf)?;
        let buf = self.d.try_decode_in_place(ctx, buf)?;
        let _ = self.o.try_decode_in_place(ctx, buf)?;

        Ok(())
    }
}

/// PLONK polynomial expression representation with its selectors and witnesses.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Polynomial {
    /// Selectors of the polynomial
    pub selectors: Selectors,
    /// Wired witnesses
    pub witnesses: WiredWitnesses,
    /// Polynomial evaluated to zero?
    pub evaluation: bool,
}

impl Element for Polynomial {
    fn len(ctx: &Config) -> usize {
        Selectors::len(ctx) + WiredWitnesses::len(ctx) + bool::len(ctx)
    }

    fn validate(&self, preamble: &Preamble) -> io::Result<()> {
        self.selectors.validate(preamble)?;
        self.witnesses.validate(preamble)?;
        self.evaluation.validate(preamble)?;

        Ok(())
    }
}

impl EncodableElement for Polynomial {
    fn to_buffer(&self, ctx: &mut EncoderContext, buf: &mut [u8]) {
        let buf = self.selectors.encode(ctx, buf);
        let buf = self.witnesses.encode(ctx, buf);
        let _ = self.evaluation.encode(ctx, buf);
    }
}

impl DecodableElement for Polynomial {
    fn try_from_buffer_in_place<'a, 'b>(
        &'a mut self,
        ctx: &DecoderContext<'a>,
        buf: &'b [u8],
    ) -> io::Result<()> {
        let buf = self.selectors.try_decode_in_place(ctx, buf)?;
        let buf = self.witnesses.try_decode_in_place(ctx, buf)?;
        let _ = self.evaluation.try_decode_in_place(ctx, buf)?;

        Ok(())
    }
}

impl Polynomial {
    /// Create a new polynomial with evaluation to either correct or incorrect
    pub const fn new(selectors: Selectors, witnesses: WiredWitnesses, evaluation: bool) -> Self {
        Self {
            selectors,
            witnesses,
            evaluation,
        }
    }

    /// Check if the polynomial evaluation is ok
    pub const fn is_ok(&self) -> bool {
        self.evaluation
    }

    /// Wire selectors
    pub const fn selectors(&self) -> &Selectors {
        &self.selectors
    }

    /// Wired witnesses
    pub const fn witnesses(&self) -> &WiredWitnesses {
        &self.witnesses
    }
}
