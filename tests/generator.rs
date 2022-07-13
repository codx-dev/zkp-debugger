use dusk_plonk_debugger::cdf::*;

use rand::distributions::Alphanumeric;
use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};

pub struct CDFGenerator {
    rng: StdRng,
}

impl CDFGenerator {
    pub fn new(seed: u64) -> Self {
        Self::seed_from_u64(seed)
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

    pub fn gen_valid_indexed_witness(
        &mut self,
        preamble: &Preamble,
        witnesses: &[Witness],
    ) -> IndexedWitness {
        let id = self.gen_range(0..preamble.witnesses());
        let origin = if self.gen() {
            Some(self.gen_range(0..preamble.constraints()))
        } else {
            None
        };
        let value = *witnesses[id as usize].value();

        IndexedWitness::new(id, origin, value)
    }

    pub fn gen_structurally_sound_circuit(
        &mut self,
        preamble: &Preamble,
    ) -> (Vec<Witness>, Vec<Constraint>) {
        let witnesses: Vec<Witness> = (0..preamble.witnesses())
            .map(|id| {
                let value = self.gen_scalar();
                let source = self.gen_source();

                Witness::new(id, value, source)
            })
            .collect();

        let constraints = (0..preamble.constraints())
            .map(|id| {
                let qm = self.gen_scalar();
                let ql = self.gen_scalar();
                let qr = self.gen_scalar();
                let qd = self.gen_scalar();
                let qc = self.gen_scalar();
                let qo = self.gen_scalar();
                let pi = self.gen_scalar();

                let a = self.gen_valid_indexed_witness(&preamble, &witnesses);
                let b = self.gen_valid_indexed_witness(&preamble, &witnesses);
                let d = self.gen_valid_indexed_witness(&preamble, &witnesses);
                let o = self.gen_valid_indexed_witness(&preamble, &witnesses);

                let re = self.gen();

                let polynomial = Polynomial::new(qm, ql, qr, qd, qc, qo, pi, a, b, d, o, re);

                let source = self.gen_source();

                Constraint::new(id, polynomial, source)
            })
            .collect();

        (witnesses, constraints)
    }
}

impl SeedableRng for CDFGenerator {
    type Seed = <StdRng as SeedableRng>::Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            rng: StdRng::from_seed(seed),
        }
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
