use std::io;

use crate::{
    Config, DecodableElement, DecodedSource, DecoderContext, Element,
    EncodableElement, EncodableSource, EncoderContext, Polynomial, Preamble,
};

/// Analogous to [`Constraint`]. This is a constraint that can be encoded into a
/// CDF file. It implements [`EncodableElement`].
///
/// This allows the [`Encoder`](struct.Encoder.html) to encode the constraint
/// into a cdf file.
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct EncodableConstraint {
    id: usize,
    polynomial: Polynomial,
    source: EncodableSource,
}

impl EncodableConstraint {
    /// Create a new encodable constraint instance.
    pub const fn new(
        id: usize,
        polynomial: Polynomial,
        source: EncodableSource,
    ) -> Self {
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

/// Decoded constraint from a CDF file. This implements [`DecodableElement`].
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Constraint<'a> {
    id: usize,
    polynomial: Polynomial,
    source: DecodedSource<'a>,
}

impl<'a> Constraint<'a> {
    /// Constructor private to the crate because constraint is suposed to be
    /// created from the cdf file
    pub(crate) const fn _new(
        id: usize,
        polynomial: Polynomial,
        source: DecodedSource<'a>,
    ) -> Self {
        Self {
            id,
            polynomial,
            source,
        }
    }
    /// Get the id of the constraint in the constraint system.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{CircuitDescription, ZkDebugger, Breakpoint};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    ///
    /// let constraint = debugger.fetch_current_constraint()?;
    /// assert_eq!(constraint.id(), 0);
    ///
    /// # Ok(()) }
    /// ```
    pub const fn id(&self) -> usize {
        self.id
    }

    /// Get the Polynomial representation of the constraint.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{CircuitDescription, ZkDebugger, Breakpoint};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    ///
    /// let constraint = debugger.fetch_constraint(9)?;
    /// let polynomial = constraint.polynomial();
    ///
    /// assert!(polynomial.evaluation);
    ///
    /// # Ok(()) }
    /// ```
    pub const fn polynomial(&self) -> &Polynomial {
        &self.polynomial
    }

    /// Get the line of the source code where the constraint is located.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{CircuitDescription, ZkDebugger, Breakpoint};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    ///
    /// let constraint = debugger.fetch_constraint(9)?;
    /// assert_eq!(constraint.line(), 26);
    ///
    /// # Ok(()) }
    /// ```
    pub const fn line(&self) -> u64 {
        self.source.line
    }

    /// Get the column of the source code where the constraint is located.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{CircuitDescription, ZkDebugger, Breakpoint};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    ///
    /// let constraint = debugger.fetch_constraint(9)?;
    /// assert_eq!(constraint.col(), 13);
    ///
    /// # Ok(()) }
    /// ```
    pub const fn col(&self) -> u64 {
        self.source.col
    }

    /// Get the source file name as a string
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{CircuitDescription, ZkDebugger, Breakpoint};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    ///
    /// let constraint = debugger.fetch_constraint(9)?;
    /// assert_eq!(constraint.name(), "/home/vlopes/dev/codex/tmp/plonk-dbg-lib/src/main.rs");
    ///
    /// # Ok(()) }
    /// ```
    pub const fn name(&self) -> &str {
        self.source.name
    }

    /// Get the source code contents as a string.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{CircuitDescription, ZkDebugger, Breakpoint};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    ///
    /// let constraint = debugger.fetch_constraint(9)?;
    /// assert_eq!(constraint.contents().len(), 1168);
    ///
    /// # Ok(()) }
    /// ```
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
