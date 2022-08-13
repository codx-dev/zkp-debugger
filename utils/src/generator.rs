use std::io;

use dusk_cdf::*;

use rand::distributions::Alphanumeric;
use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};

pub struct CDFGenerator {
    rng: StdRng,
    preamble: Preamble,
    text_index: usize,
}

impl CDFGenerator {
    pub fn new(seed: u64, preamble: Preamble) -> Self {
        let rng = StdRng::seed_from_u64(seed);
        let text_index = 0;

        Self {
            rng,
            preamble,
            text_index,
        }
    }

    pub fn gen_cdf(&mut self) -> CircuitDescription<io::Cursor<Vec<u8>>> {
        self.gen_cdf_with_data().0
    }

    pub fn gen_cdf_with_data(
        &mut self,
    ) -> (
        CircuitDescription<io::Cursor<Vec<u8>>>,
        Vec<Witness>,
        Vec<Constraint>,
    ) {
        let (cursor, witnesses, constraints) = self.gen_cursor_with_data();

        let cdf = CircuitDescription::from_reader(cursor).expect("failed to generate cdf file");

        (cdf, witnesses, constraints)
    }

    pub fn gen_cursor(&mut self) -> io::Cursor<Vec<u8>> {
        self.gen_cursor_with_data().0
    }

    pub fn gen_cursor_with_data(&mut self) -> (io::Cursor<Vec<u8>>, Vec<Witness>, Vec<Constraint>) {
        self.gen_cursor_with_callback_with_data(|w| w, |c| c)
    }

    // clippy isn't smart enough here to understand its a callback function, so the collect is
    // needed
    #[allow(clippy::needless_collect)]
    pub fn gen_cursor_with_callback<W, C>(&mut self, w: W, c: C) -> io::Cursor<Vec<u8>>
    where
        W: FnMut(Witness) -> Witness,
        C: FnMut(Constraint) -> Constraint,
    {
        self.gen_cursor_with_callback_with_data(w, c).0
    }

    // clippy isn't smart enough here to understand its a callback function, so the collect is
    // needed
    #[allow(clippy::needless_collect)]
    pub fn gen_cursor_with_callback_with_data<W, C>(
        &mut self,
        w: W,
        c: C,
    ) -> (io::Cursor<Vec<u8>>, Vec<Witness>, Vec<Constraint>)
    where
        W: FnMut(Witness) -> Witness,
        C: FnMut(Constraint) -> Constraint,
    {
        let config = Config::DEFAULT;
        let (witnesses, constraints) = self.gen_structurally_sound_circuit();

        let witnesses: Vec<Witness> = witnesses.into_iter().map(w).collect();
        let constraints: Vec<Constraint> = constraints.into_iter().map(c).collect();

        let mut encoder = Encoder::init_cursor(
            config,
            witnesses.clone().into_iter(),
            constraints.clone().into_iter(),
        );

        encoder.write_all().expect("failed to serialize circuit");

        let mut cursor = encoder.into_inner();

        cursor.set_position(0);

        (cursor, witnesses, constraints)
    }

    pub fn gen_scalar(&mut self) -> Scalar {
        let mut scalar = [0u8; Scalar::LEN];

        self.fill_bytes(&mut scalar);

        scalar.into()
    }

    pub fn gen_fixed_text<const N: u16>(&mut self) -> FixedText<N> {
        let text: String = self
            .rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(N as usize)
            .map(char::from)
            .collect();

        let mut text = format!("text-{}-{}", self.text_index, text);
        self.text_index += 1;

        let seed = sha256::digest_bytes(text.as_bytes()).as_bytes()[..32]
            .try_into()
            .expect("failed to generate seed");

        self.rng = StdRng::from_seed(seed);

        // Input bigger than expected should be truncated
        let n = self.gen_range(0..N * 2) as usize;
        text.truncate(n);

        text.into()
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

    pub fn gen_witness(&mut self) -> Witness {
        let id = self.gen();
        let value = self.gen_scalar();
        let source = self.gen_source();

        Witness::new(id, value, source)
    }

    pub fn gen_valid_indexed_witness(&mut self, witnesses: &[Witness]) -> IndexedWitness {
        let id = self.gen_range(0..self.preamble.witnesses);
        let origin = if self.gen() {
            Some(self.gen_range(0..self.preamble.constraints))
        } else {
            None
        };
        let value = *witnesses[id].value();

        IndexedWitness::new(id, origin, value)
    }

    pub fn gen_structurally_sound_circuit(&mut self) -> (Vec<Witness>, Vec<Constraint>) {
        let witnesses: Vec<Witness> = (0..self.preamble.witnesses)
            .map(|id| {
                let value = self.gen_scalar();
                let source = self.gen_source();

                Witness::new(id, value, source)
            })
            .collect();

        let constraints = (0..self.preamble.constraints)
            .map(|id| {
                let qm = self.gen_scalar();
                let ql = self.gen_scalar();
                let qr = self.gen_scalar();
                let qd = self.gen_scalar();
                let qc = self.gen_scalar();
                let qo = self.gen_scalar();
                let pi = self.gen_scalar();
                let qarith = self.gen_scalar();
                let qlogic = self.gen_scalar();
                let qvariable_add = self.gen_scalar();

                let a = self.gen_valid_indexed_witness(&witnesses);
                let b = self.gen_valid_indexed_witness(&witnesses);
                let d = self.gen_valid_indexed_witness(&witnesses);
                let o = self.gen_valid_indexed_witness(&witnesses);

                let re = self.gen();

                let polynomial = Polynomial::new(
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
                );

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
fn generator_is_rng() {
    let mut generator = CDFGenerator::new(
        0x348,
        *Preamble::new().with_witnesses(1).with_constraints(0),
    );

    let mut bytes = vec![0xfa; 32];

    generator
        .try_fill_bytes(&mut bytes)
        .expect("failed to fill bytes");
}

#[test]
fn generate_circuit_is_valid_cdf() {
    use std::collections::HashSet;

    let cases = vec![
        *Preamble::new().with_witnesses(1).with_constraints(0),
        *Preamble::new().with_witnesses(1).with_constraints(1),
        *Preamble::new().with_witnesses(1).with_constraints(10),
        *Preamble::new().with_witnesses(10).with_constraints(0),
        *Preamble::new().with_witnesses(10).with_constraints(10),
        *Preamble::new().with_witnesses(10).with_constraints(100),
        *Preamble::new().with_witnesses(100).with_constraints(10),
    ];

    for preamble in cases {
        let w_len = preamble.witnesses;
        let c_len = preamble.constraints;

        let mut generator = CDFGenerator::new(0x348, preamble);

        let (witnesses, constraints) = generator.gen_structurally_sound_circuit();

        let source_cache: HashSet<FixedText<{ Source::PATH_LEN }>> = witnesses
            .iter()
            .map(|w| w.source().path().clone())
            .chain(constraints.iter().map(|c| c.source().path().clone()))
            .collect();

        let source_cache_len = source_cache.len();

        assert_eq!(w_len, witnesses.len());
        assert_eq!(c_len, constraints.len());

        let n = Encoder::init_cursor(
            preamble.config,
            witnesses.into_iter(),
            constraints.into_iter(),
        )
        .write_all()
        .expect("failed to serialize circuit");

        assert_eq!(
            n,
            Preamble::LEN
                + w_len * Witness::len(&preamble.config)
                + c_len * Constraint::len(&preamble.config)
                + source_cache_len * Source::PATH_LEN as usize
        );

        let (mut cdf, witnesses, constraints) = generator.gen_cdf_with_data();

        witnesses.into_iter().enumerate().for_each(|(idx, w)| {
            let w_p = cdf.fetch_witness(idx).expect("failed to fecth witness");

            assert_eq!(w, w_p);
        });

        constraints.into_iter().enumerate().for_each(|(idx, c)| {
            let c_p = cdf
                .fetch_constraint(idx)
                .expect("failed to fecth constraint");

            assert_eq!(c, c_p);
        });
    }
}
