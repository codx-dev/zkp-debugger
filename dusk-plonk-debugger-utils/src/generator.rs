use std::io;

use dusk_plonk_cdf::*;

use rand::distributions::Alphanumeric;
use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};

pub struct CDFGenerator {
    rng: StdRng,
    preamble: Preamble,
}

impl CDFGenerator {
    pub fn new(seed: u64, preamble: Preamble) -> Self {
        let rng = StdRng::seed_from_u64(seed);

        Self { rng, preamble }
    }

    pub fn gen_cursor(&mut self) -> io::Cursor<Vec<u8>> {
        self.gen_cursor_with_callback(|w| w, |c| c)
    }

    // clippy isn't smart enough here to understand its a callback function, so the collect is
    // needed
    #[allow(clippy::needless_collect)]
    pub fn gen_cursor_with_callback<W, C>(&mut self, w: W, c: C) -> io::Cursor<Vec<u8>>
    where
        W: FnMut(Witness) -> Witness,
        C: FnMut(Constraint) -> Constraint,
    {
        let mut cursor = io::Cursor::new(Vec::new());

        let (witnesses, constraints) = self.gen_structurally_sound_circuit();

        let witnesses: Vec<Witness> = witnesses.into_iter().map(w).collect();
        let constraints: Vec<Constraint> = constraints.into_iter().map(c).collect();

        CircuitDescriptionUnit::write_all(
            &mut cursor,
            witnesses.into_iter(),
            constraints.into_iter(),
        )
        .expect("failed to serialize circuit");

        cursor.set_position(0);

        cursor
    }

    pub fn gen_scalar(&mut self) -> Scalar {
        let mut scalar = [0u8; Scalar::LEN];

        self.fill_bytes(&mut scalar);

        scalar.into()
    }

    pub fn gen_fixed_text<const N: u16>(&mut self) -> FixedText<N> {
        let mut text: String = self
            .rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(N as usize)
            .map(char::from)
            .collect();

        let seed = sha256::digest_bytes(text.as_bytes()).as_bytes()[..32]
            .try_into()
            .expect("failed to generate seed");

        self.rng = StdRng::from_seed(seed);

        // Input bigger than expected should be truncated
        let n = self.gen_range(0..N * 2) as usize;
        text.truncate(n);

        text.into()
    }

    pub fn gen_preamble(&mut self) -> Preamble {
        Preamble::new(self.gen(), self.gen())
    }

    pub fn gen_indexed_witness(&mut self) -> IndexedWitness {
        let index = self.gen();
        let origin = if self.gen() { Some(self.gen()) } else { None };
        let value = self.gen_scalar();

        IndexedWitness::new(index, origin, value)
    }

    pub fn gen_polynomial(&mut self) -> Polynomial {
        Polynomial::new(
            self.gen_scalar(),
            self.gen_scalar(),
            self.gen_scalar(),
            self.gen_scalar(),
            self.gen_scalar(),
            self.gen_scalar(),
            self.gen_scalar(),
            self.gen_indexed_witness(),
            self.gen_indexed_witness(),
            self.gen_indexed_witness(),
            self.gen_indexed_witness(),
            self.gen(),
        )
    }

    pub fn gen_source(&mut self) -> Source {
        let line = self.gen();
        let col = self.gen();
        let path = self.gen_fixed_text();

        Source::new(line, col, path)
    }

    pub fn gen_constraint(&mut self) -> Constraint {
        let id = self.gen();
        let polynomial = self.gen_polynomial();
        let source = self.gen_source();

        Constraint::new(id, polynomial, source)
    }

    pub fn gen_witness(&mut self) -> Witness {
        let id = self.gen();
        let value = self.gen_scalar();
        let source = self.gen_source();

        Witness::new(id, value, source)
    }

    pub fn gen_valid_indexed_witness(&mut self, witnesses: &[Witness]) -> IndexedWitness {
        let id = self.gen_range(0..self.preamble.witnesses());
        let origin = if self.gen() {
            Some(self.gen_range(0..self.preamble.constraints()))
        } else {
            None
        };
        let value = *witnesses[id as usize].value();

        IndexedWitness::new(id, origin, value)
    }

    pub fn gen_structurally_sound_circuit(&mut self) -> (Vec<Witness>, Vec<Constraint>) {
        let witnesses: Vec<Witness> = (0..self.preamble.witnesses())
            .map(|id| {
                let value = self.gen_scalar();
                let source = self.gen_source();

                Witness::new(id, value, source)
            })
            .collect();

        let constraints = (0..self.preamble.constraints())
            .map(|id| {
                let qm = self.gen_scalar();
                let ql = self.gen_scalar();
                let qr = self.gen_scalar();
                let qd = self.gen_scalar();
                let qc = self.gen_scalar();
                let qo = self.gen_scalar();
                let pi = self.gen_scalar();

                let a = self.gen_valid_indexed_witness(&witnesses);
                let b = self.gen_valid_indexed_witness(&witnesses);
                let d = self.gen_valid_indexed_witness(&witnesses);
                let o = self.gen_valid_indexed_witness(&witnesses);

                let re = self.gen();

                let polynomial = Polynomial::new(qm, ql, qr, qd, qc, qo, pi, a, b, d, o, re);

                let source = self.gen_source();

                Constraint::new(id, polynomial, source)
            })
            .collect();

        (witnesses, constraints)
    }
}

impl RngCore for CDFGenerator {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.rng.try_fill_bytes(dest)
    }
}

#[test]
fn generate_circuit_is_valid_cdf() {
    let cases = vec![
        Preamble::new(1, 0),
        Preamble::new(1, 1),
        Preamble::new(1, 10),
        Preamble::new(10, 0),
        Preamble::new(10, 10),
        Preamble::new(10, 100),
        Preamble::new(100, 10),
    ];

    for preamble in cases {
        let w_len = preamble.witnesses() as usize;
        let c_len = preamble.constraints() as usize;

        let (witnesses, constraints) =
            CDFGenerator::new(0x348, preamble).gen_structurally_sound_circuit();

        assert_eq!(w_len, witnesses.len());
        assert_eq!(c_len, constraints.len());

        let mut cursor = io::Cursor::new(Vec::new());

        let n = CircuitDescriptionUnit::write_all(
            &mut cursor,
            witnesses.clone().into_iter(),
            constraints.clone().into_iter(),
        )
        .expect("failed to generate valid CDF");

        assert_eq!(
            n,
            Preamble::LEN + w_len * Witness::len(&preamble) + c_len * Constraint::len(&preamble)
        );
    }
}
