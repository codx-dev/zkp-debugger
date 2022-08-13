use std::io;

use crate::{
    AtomicConfig, Config, Context, ContextUnit, Element, IndexedWitness, Preamble, Scalar,
};

/// PLONK polynomial expression representation with its selectors and witnesses.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Polynomial {
    qm: Scalar,
    ql: Scalar,
    qr: Scalar,
    qd: Scalar,
    qc: Scalar,
    qo: Scalar,
    pi: Scalar,
    qarith: Scalar,
    qlogic: Scalar,
    qvariable_add: Scalar,
    a: IndexedWitness,
    b: IndexedWitness,
    d: IndexedWitness,
    o: IndexedWitness,
    re: bool,
}

impl Polynomial {
    // TODO refactor into type, as clippy suggests
    #[allow(clippy::too_many_arguments)]
    /// Create a new polynomial
    pub const fn new(
        qm: Scalar,
        ql: Scalar,
        qr: Scalar,
        qd: Scalar,
        qc: Scalar,
        qo: Scalar,
        pi: Scalar,
        qarith: Scalar,
        qlogic: Scalar,
        qvariable_add: Scalar,
        a: IndexedWitness,
        b: IndexedWitness,
        d: IndexedWitness,
        o: IndexedWitness,
        re: bool,
    ) -> Self {
        Self {
            qm,
            ql,
            qr,
            qd,
            qc,
            qo,
            pi,
            qarith,
            qlogic,
            qvariable_add,
            a,
            b,
            d,
            o,
            re,
        }
    }

    /// Check if the polynomial evaluation is ok
    pub const fn is_ok(&self) -> bool {
        self.re
    }

    // TODO refactor into type, as clippy suggests
    #[allow(clippy::type_complexity)]
    /// Fetch the constraint internals
    pub const fn internals(
        &self,
    ) -> (
        &Scalar,
        &Scalar,
        &Scalar,
        &Scalar,
        &Scalar,
        &Scalar,
        &Scalar,
        &Scalar,
        &Scalar,
        &Scalar,
        &IndexedWitness,
        &IndexedWitness,
        &IndexedWitness,
        &IndexedWitness,
        bool,
    ) {
        (
            &self.qm,
            &self.ql,
            &self.qr,
            &self.qd,
            &self.qc,
            &self.qo,
            &self.pi,
            &self.qarith,
            &self.qlogic,
            &self.qvariable_add,
            &self.a,
            &self.b,
            &self.d,
            &self.o,
            self.re,
        )
    }
}

impl Element for Polynomial {
    type Config = Config;

    fn zeroed() -> Self {
        Self::default()
    }

    fn len(config: &Self::Config) -> usize {
        10 * Scalar::len(config) + 4 * IndexedWitness::len(config) + bool::len(&AtomicConfig)
    }

    fn to_buffer(&self, config: &Self::Config, context: &mut ContextUnit, buf: &mut [u8]) {
        let buf = self.qm.encode(config, context, buf);
        let buf = self.ql.encode(config, context, buf);
        let buf = self.qr.encode(config, context, buf);
        let buf = self.qd.encode(config, context, buf);
        let buf = self.qc.encode(config, context, buf);
        let buf = self.qo.encode(config, context, buf);
        let buf = self.pi.encode(config, context, buf);
        let buf = self.qarith.encode(config, context, buf);
        let buf = self.qlogic.encode(config, context, buf);
        let buf = self.qvariable_add.encode(config, context, buf);
        let buf = self.a.encode(config, context, buf);
        let buf = self.b.encode(config, context, buf);
        let buf = self.d.encode(config, context, buf);
        let buf = self.o.encode(config, context, buf);
        let _ = self.re.encode(&AtomicConfig, context, buf);
    }

    fn try_from_buffer_in_place<S>(
        &mut self,
        config: &Self::Config,
        context: &mut Context<S>,
        buf: &[u8],
    ) -> io::Result<()>
    where
        S: io::Read + io::Seek,
    {
        Self::validate_buffer_len(config, buf.len())?;

        let buf = self.qm.try_decode_in_place(config, context, buf)?;
        let buf = self.ql.try_decode_in_place(config, context, buf)?;
        let buf = self.qr.try_decode_in_place(config, context, buf)?;
        let buf = self.qd.try_decode_in_place(config, context, buf)?;
        let buf = self.qc.try_decode_in_place(config, context, buf)?;
        let buf = self.qo.try_decode_in_place(config, context, buf)?;
        let buf = self.pi.try_decode_in_place(config, context, buf)?;
        let buf = self.qarith.try_decode_in_place(config, context, buf)?;
        let buf = self.qlogic.try_decode_in_place(config, context, buf)?;
        let buf = self
            .qvariable_add
            .try_decode_in_place(config, context, buf)?;
        let buf = self.a.try_decode_in_place(config, context, buf)?;
        let buf = self.b.try_decode_in_place(config, context, buf)?;
        let buf = self.d.try_decode_in_place(config, context, buf)?;
        let buf = self.o.try_decode_in_place(config, context, buf)?;
        let _ = self.re.try_decode_in_place(&AtomicConfig, context, buf)?;

        Ok(())
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
        self.qvariable_add.validate(preamble)?;
        self.a.validate(preamble)?;
        self.b.validate(preamble)?;
        self.d.validate(preamble)?;
        self.o.validate(preamble)?;
        self.re.validate(preamble)?;

        Ok(())
    }
}

#[test]
fn validate_works() {
    Polynomial::zeroed()
        .validate(&Default::default())
        .expect("default config validate should pass");
}
