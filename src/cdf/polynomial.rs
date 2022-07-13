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
}

impl Element for Polynomial {
    const LEN: usize = 7 * Scalar::LEN + 4 * IndexedWitness::LEN + bool::LEN;

    fn zeroed() -> Self {
        Self::default()
    }

    fn to_buffer(&self, buf: &mut [u8]) {
        let buf = self.qm.encode(buf);
        let buf = self.ql.encode(buf);
        let buf = self.qr.encode(buf);
        let buf = self.qd.encode(buf);
        let buf = self.qc.encode(buf);
        let buf = self.qo.encode(buf);
        let buf = self.pi.encode(buf);
        let buf = self.a.encode(buf);
        let buf = self.b.encode(buf);
        let buf = self.d.encode(buf);
        let buf = self.o.encode(buf);
        let _ = self.re.encode(buf);
    }

    fn try_from_buffer_in_place(&mut self, buf: &[u8]) -> io::Result<()> {
        let buf = self.qm.try_decode_in_place(buf)?;
        let buf = self.ql.try_decode_in_place(buf)?;
        let buf = self.qr.try_decode_in_place(buf)?;
        let buf = self.qd.try_decode_in_place(buf)?;
        let buf = self.qc.try_decode_in_place(buf)?;
        let buf = self.qo.try_decode_in_place(buf)?;
        let buf = self.pi.try_decode_in_place(buf)?;
        let buf = self.a.try_decode_in_place(buf)?;
        let buf = self.b.try_decode_in_place(buf)?;
        let buf = self.d.try_decode_in_place(buf)?;
        let buf = self.o.try_decode_in_place(buf)?;
        let _ = self.re.try_decode_in_place(buf)?;

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

impl io::Write for Polynomial {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.try_write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.qm.flush()?;
        self.ql.flush()?;
        self.qr.flush()?;
        self.qd.flush()?;
        self.qc.flush()?;
        self.qo.flush()?;
        self.pi.flush()?;
        self.a.flush()?;
        self.b.flush()?;
        self.d.flush()?;
        self.o.flush()?;

        Ok(())
    }
}

impl io::Read for Polynomial {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.try_read(buf)
    }
}
