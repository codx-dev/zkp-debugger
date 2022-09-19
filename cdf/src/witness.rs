use std::io;

use crate::{
    Config, DecodableElement, DecodedSource, DecoderContext, Element,
    EncodableElement, EncodableSource, EncoderContext, Preamble, Scalar,
};

/// Analogous to [`Witness`]. This is a witness that can be encoded into a
/// CDF file. It implements [`EncodableElement`].
///
/// This allows the [`Encoder`](struct.Encoder.html) to encode the constraint
/// into a cdf file.
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

    /// Source reference to be encoded
    pub const fn source(&self) -> &EncodableSource {
        &self.source
    }
}

impl Element for EncodableWitness {
    fn len(ctx: &Config) -> usize {
        usize::len(ctx)
            + <Option<usize>>::len(ctx)
            + Scalar::len(ctx)
            + EncodableSource::len(ctx)
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

/// Witness decoded from a CDF file. This implements [`DecodableElement`].
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Witness<'a> {
    id: usize,
    constraint: Option<usize>,
    value: Scalar,
    source: DecodedSource<'a>,
}

impl<'a> Witness<'a> {
    /// Constructor private to the crate because witness is suposed to be
    /// created from the cdf file
    pub(crate) const fn _new(
        id: usize,
        constraint: Option<usize>,
        value: Scalar,
        source: DecodedSource<'a>,
    ) -> Self {
        Self {
            id,
            constraint,
            value,
            source,
        }
    }

    /// Id of the witness in the constraint system
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::ZkDebugger;
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    /// let witness = debugger.fetch_witness(0)?;
    ///
    /// assert_eq!(witness.id(), 0);
    ///
    /// # Ok(()) }
    /// ```
    pub const fn id(&self) -> usize {
        self.id
    }

    /// Constraint that originated the witness
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::ZkDebugger;
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    /// let witness = debugger.fetch_witness(4)?;
    ///
    /// assert_eq!(witness.constraint(), None);
    ///
    /// # Ok(()) }
    /// ```
    pub const fn constraint(&self) -> Option<usize> {
        self.constraint
    }

    /// Value of the witness in the constraint system
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{Scalar, ZkDebugger};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    /// let witness = debugger.fetch_witness(4)?;
    /// let mut value = Scalar::default();
    /// value[0] = 7;
    ///
    /// assert_eq!(*witness.value(), value);
    ///
    /// # Ok(()) }
    /// ```
    pub const fn value(&self) -> &Scalar {
        &self.value
    }

    /// Line of the source code of the witness
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{Scalar, ZkDebugger};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    /// let witness = debugger.fetch_witness(4)?;
    ///
    /// assert_eq!(witness.line(), 33);
    ///
    /// # Ok(()) }
    /// ```
    pub const fn line(&self) -> u64 {
        self.source.line
    }

    /// Get the column of the source code
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{Scalar, ZkDebugger};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    /// let witness = debugger.fetch_witness(4)?;
    ///
    /// assert_eq!(witness.col(), 34);
    ///
    /// # Ok(()) }
    /// ```
    pub const fn col(&self) -> u64 {
        self.source.col
    }

    /// Source file name
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{Scalar, ZkDebugger};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    /// let witness = debugger.fetch_witness(4)?;
    ///
    /// assert_eq!(witness.name(), "/home/vlopes/dev/codex/tmp/plonk-dbg-lib/src/main.rs");
    ///
    /// # Ok(()) }
    /// ```
    pub const fn name(&self) -> &str {
        self.source.name
    }

    /// Source code contents
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{Scalar, ZkDebugger};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    /// let witness = debugger.fetch_witness(2)?;
    ///
    /// assert_eq!(witness.contents().len(), 1168);
    ///
    /// # Ok(()) }
    /// ```
    pub const fn contents(&self) -> &str {
        self.source.contents
    }
}

impl<'a> Element for Witness<'a> {
    fn len(ctx: &Config) -> usize {
        usize::len(ctx)
            + <Option<usize>>::len(ctx)
            + Scalar::len(ctx)
            + DecodedSource::len(ctx)
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

        let buf = self.id.try_decode_in_place(ctx, buf)?;
        let buf = self.constraint.try_decode_in_place(ctx, buf)?;
        let buf = self.value.try_decode_in_place(ctx, buf)?;
        let _ = self.source.try_decode_in_place(ctx, buf)?;

        Ok(())
    }
}
