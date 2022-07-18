use std::io;

use super::{Element, IndexedWitness, Preamble, Scalar};

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
    a: IndexedWitness,
    b: IndexedWitness,
    d: IndexedWitness,
    o: IndexedWitness,
    re: bool,
}

impl Polynomial {
    // Selectors contain multiple - but fixed - circuit definitions. Its not desirable to have
    // error-prone variable len structs such as vec because these might be misused
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
        &IndexedWitness,
        &IndexedWitness,
        &IndexedWitness,
        &IndexedWitness,
        bool,
    ) {
        (
            &self.qm, &self.ql, &self.qr, &self.qd, &self.qc, &self.qo, &self.pi, &self.a, &self.b,
            &self.d, &self.o, self.re,
        )
    }
}

impl Element for Polynomial {
    fn zeroed() -> Self {
        Self::default()
    }

    fn len(preamble: &Preamble) -> usize {
        7 * Scalar::len(preamble) + 4 * IndexedWitness::len(preamble) + bool::len(preamble)
    }

    fn to_buffer(&self, preamble: &Preamble, buf: &mut [u8]) {
        let buf = self.qm.encode(preamble, buf);
        let buf = self.ql.encode(preamble, buf);
        let buf = self.qr.encode(preamble, buf);
        let buf = self.qd.encode(preamble, buf);
        let buf = self.qc.encode(preamble, buf);
        let buf = self.qo.encode(preamble, buf);
        let buf = self.pi.encode(preamble, buf);
        let buf = self.a.encode(preamble, buf);
        let buf = self.b.encode(preamble, buf);
        let buf = self.d.encode(preamble, buf);
        let buf = self.o.encode(preamble, buf);
        let _ = self.re.encode(preamble, buf);
    }

    fn try_from_buffer_in_place(&mut self, preamble: &Preamble, buf: &[u8]) -> io::Result<()> {
        let buf = self.qm.try_decode_in_place(preamble, buf)?;
        let buf = self.ql.try_decode_in_place(preamble, buf)?;
        let buf = self.qr.try_decode_in_place(preamble, buf)?;
        let buf = self.qd.try_decode_in_place(preamble, buf)?;
        let buf = self.qc.try_decode_in_place(preamble, buf)?;
        let buf = self.qo.try_decode_in_place(preamble, buf)?;
        let buf = self.pi.try_decode_in_place(preamble, buf)?;
        let buf = self.a.try_decode_in_place(preamble, buf)?;
        let buf = self.b.try_decode_in_place(preamble, buf)?;
        let buf = self.d.try_decode_in_place(preamble, buf)?;
        let buf = self.o.try_decode_in_place(preamble, buf)?;
        let _ = self.re.try_decode_in_place(preamble, buf)?;

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
        self.a.validate(preamble)?;
        self.b.validate(preamble)?;
        self.d.validate(preamble)?;
        self.o.validate(preamble)?;
        self.re.validate(preamble)?;

        Ok(())
    }
}
