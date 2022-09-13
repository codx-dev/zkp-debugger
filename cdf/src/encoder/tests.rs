use std::borrow::Borrow;
use std::collections::HashMap;
use std::iter;

use crate::*;
use quickcheck::{quickcheck, Arbitrary, Gen, TestResult};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use sha2::Digest;

// hard limit to prevent huge sets from being generated
//
// not using `Gen::size` so we define our own limit
const LIMIT: usize = 25;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneratedSource {
    pub source: EncodableSource,
    pub contents: String,
}

impl Arbitrary for GeneratedSource {
    fn arbitrary(g: &mut Gen) -> Self {
        let lines = usize::arbitrary(g) % LIMIT;

        let line = 1u64.saturating_add(u64::arbitrary(g) % lines.max(1) as u64);
        let col = u64::arbitrary(g);

        let rng = u64::arbitrary(g);
        let rng = &mut StdRng::seed_from_u64(rng);

        let contents = (0..lines).fold(
            String::with_capacity(lines * u8::MAX as usize),
            |mut s, _| {
                let cols = u8::arbitrary(g) as usize;
                let contents = rng
                    .sample_iter::<char, _>(rand::distributions::Standard)
                    .take(cols)
                    .chain(iter::once('\n'));

                s.extend(contents);
                s
            },
        );

        let path = sha2::Sha256::digest(&contents);
        let path = hex::encode(path);

        let source = EncodableSource::new(line, col, path.into());

        Self { source, contents }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneratedWitness {
    pub witness: EncodableWitness,
    pub contents: String,
}

impl Arbitrary for GeneratedWitness {
    fn arbitrary(g: &mut Gen) -> Self {
        let id = 0;
        let constraint = None;
        let value = Scalar::arbitrary(g);
        let GeneratedSource { source, contents } = GeneratedSource::arbitrary(g);

        let witness = EncodableWitness::new(id, constraint, value, source);

        Self { witness, contents }
    }
}

impl Borrow<EncodableWitness> for GeneratedWitness {
    fn borrow(&self) -> &EncodableWitness {
        &self.witness
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneratedWitnesses {
    pub witnesses: Vec<GeneratedWitness>,
}

impl Arbitrary for GeneratedWitnesses {
    fn arbitrary(g: &mut Gen) -> Self {
        let count = usize::arbitrary(g) % LIMIT;
        let witnesses = (0..count).map(|_| GeneratedWitness::arbitrary(g)).collect();

        Self { witnesses }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneratedConstraint {
    pub constraint: EncodableConstraint,
    pub contents: String,
}

impl Arbitrary for GeneratedConstraint {
    fn arbitrary(g: &mut Gen) -> Self {
        let id = 0;
        let polynomial = Polynomial::arbitrary(g);
        let GeneratedSource { source, contents } = GeneratedSource::arbitrary(g);

        let constraint = EncodableConstraint::new(id, polynomial, source);

        Self {
            constraint,
            contents,
        }
    }
}

impl Borrow<EncodableConstraint> for GeneratedConstraint {
    fn borrow(&self) -> &EncodableConstraint {
        &self.constraint
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneratedConstraints {
    pub constraints: Vec<GeneratedConstraint>,
}

impl Arbitrary for GeneratedConstraints {
    fn arbitrary(g: &mut Gen) -> Self {
        let count = usize::arbitrary(g) % LIMIT;
        let constraints = (0..count)
            .map(|_| GeneratedConstraint::arbitrary(g))
            .collect();

        Self { constraints }
    }
}

fn prop(
    seed: u64,
    config: Config,
    witnesses: GeneratedWitnesses,
    constraints: GeneratedConstraints,
) -> TestResult {
    let witnesses = witnesses.witnesses;
    let constraints = constraints.constraints;

    // a circuit containts at least one witness
    if witnesses.is_empty() {
        return TestResult::discard();
    }

    // discard the unlikely event of duplicated generated file contents
    let disk: HashMap<String, String> = witnesses
        .iter()
        .map(|w| (w.witness.source().path().to_string(), w.contents.clone()))
        .chain(
            constraints
                .iter()
                .map(|c| (c.constraint.source().path().to_string(), c.contents.clone())),
        )
        .collect();

    let rng = &mut StdRng::seed_from_u64(seed);

    // witness is now an ordered set with randomly associated constraints
    let witnesses: Vec<GeneratedWitness> = witnesses
        .into_iter()
        .enumerate()
        .map(|(id, generated)| {
            let GeneratedWitness { witness, contents } = generated;

            let constraint = if !constraints.is_empty() && rng.gen() {
                Some(rng.gen_range(0..constraints.len()))
            } else {
                None
            };

            let value = *witness.value();
            let source = witness.source().clone();

            let witness = EncodableWitness::new(id, constraint, value, source);

            GeneratedWitness { witness, contents }
        })
        .collect();

    let constraints: Vec<GeneratedConstraint> = constraints
        .into_iter()
        .enumerate()
        .map(|(id, generated)| {
            let GeneratedConstraint {
                constraint,
                contents,
            } = generated;

            let source = constraint.source().clone();

            let mut polynomial = constraint.polynomial().clone();

            // wire polynomial to the generated witnesses
            polynomial.witnesses = WiredWitnesses {
                a: polynomial.witnesses.a % witnesses.len(),
                b: polynomial.witnesses.b % witnesses.len(),
                d: polynomial.witnesses.d % witnesses.len(),
                o: polynomial.witnesses.o % witnesses.len(),
            };

            let constraint = EncodableConstraint::new(id, polynomial, source);

            GeneratedConstraint {
                constraint,
                contents,
            }
        })
        .collect();

    let mut encoder = Encoder::init_cursor(
        config,
        witnesses.clone().into_iter(),
        constraints.clone().into_iter(),
    );

    if let Err(e) = encoder.write_all(disk) {
        return TestResult::error(format!("{}", e));
    }

    let cursor = encoder.into_inner();

    let mut circuit = match CircuitDescription::from_reader(cursor) {
        Ok(c) => c,
        Err(e) => return TestResult::error(format!("{}", e)),
    };

    for witness in witnesses {
        let GeneratedWitness { witness, contents } = witness;

        let w = match circuit.fetch_witness(witness.id()) {
            Ok(w) => w,
            Err(e) => return TestResult::error(format!("{}", e)),
        };

        let line = witness.source().line();
        let col = witness.source().col();
        let name = witness.source().path();
        let contents = contents.as_str();

        let source = DecodedSource {
            line,
            col,
            name,
            contents,
        };

        let value = config
            .zeroed_scalar_values
            .then_some(Scalar::default())
            .unwrap_or_else(|| *witness.value());

        let witness = Witness::_new(witness.id(), witness.constraint(), value, source);

        if w != witness {
            return TestResult::error("unexpected decoded witness");
        }
    }

    for constraint in constraints {
        let GeneratedConstraint {
            constraint,
            contents,
        } = constraint;

        let c = match circuit.fetch_constraint(constraint.id()) {
            Ok(c) => c,
            Err(e) => return TestResult::error(format!("{}", e)),
        };

        let mut polynomial = constraint.polynomial().clone();

        if config.zeroed_scalar_values {
            polynomial.selectors = Selectors::default();
        }

        let line = constraint.source().line();
        let col = constraint.source().col();
        let name = constraint.source().path();
        let contents = contents.as_str();

        let source = DecodedSource {
            line,
            col,
            name,
            contents,
        };

        let constraint = Constraint::_new(constraint.id(), polynomial, source);

        if c != constraint {
            return TestResult::error("unexpected decoded constraint");
        }
    }

    TestResult::passed()
}

#[test]
fn encode_decode_works() {
    quickcheck(prop as fn(_, _, _, _) -> _);
}
