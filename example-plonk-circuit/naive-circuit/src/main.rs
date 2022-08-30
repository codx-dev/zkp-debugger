use std::path::PathBuf;
use std::{env, fs};

use clap::Parser;
use dusk_plonk::prelude::*;

/// Naive sign-verify circuit
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Secret key
    #[clap(short, long, value_parser, default_value_t = 65)]
    key: u64,
}

#[derive(Debug, Default, Clone, Copy)]
struct NaiveSignature {
    key: BlsScalar,
    signature: BlsScalar,
    input: [BlsScalar; Self::COUNT],
}

impl NaiveSignature {
    const COUNT: usize = 7;

    pub fn input() -> [BlsScalar; Self::COUNT] {
        let mut input = [BlsScalar::zero(); Self::COUNT];

        input.iter_mut().fold(BlsScalar::from(3), |acc, i| {
            *i = acc;
            acc * BlsScalar::from(2)
        });

        input
    }
}

impl Circuit for NaiveSignature {
    fn circuit<C>(&self, composer: &mut C) -> Result<(), Error>
    where
        C: Composer,
    {
        let key = composer.append_public(self.key);
        let signature = composer.append_public(self.signature);

        let mut input = [C::ZERO; Self::COUNT];

        input
            .iter_mut()
            .zip(self.input.iter())
            .for_each(|(w, i)| *w = composer.append_witness(*i));

        naive_signature::gadget::verify(composer, key, signature, &input);

        let sig = naive_signature::gadget::sign(composer, key, &input);

        composer.assert_equal(sig, signature);

        Ok(())
    }
}

fn main() {
    let Args { key } = Args::parse();

    let key = BlsScalar::from(key);
    let input = NaiveSignature::input();
    let signature = naive_signature::native::sign(&key, &input);

    // sanity check
    assert!(naive_signature::native::verify(&key, &signature, &input));

    let cdf = env::var("CDF_OUTPUT")
        .expect("the target CDF output path (`CDF_OUTPUT`) environment variable is not set!");

    let cdf = PathBuf::from(cdf);

    if fs::remove_file(&cdf).is_ok() {
        println!("CDF file removed, generating...");
    }

    let label = b"transcript-arguments";
    let pp = PublicParameters::setup(1 << 8, &mut rand::thread_rng()).expect("failed to setup");

    let (prover, verifier) =
        Compiler::compile::<NaiveSignature>(&pp, label).expect("failed to compile circuit");

    let circuit = NaiveSignature {
        key,
        signature,
        input,
    };

    // Generate the proof and its public inputs
    let (proof, public_inputs) = prover
        .prove(&mut rand::thread_rng(), &circuit)
        .expect("failed to prove");

    println!(
        "proof evaluation: {}",
        verifier.verify(&proof, &public_inputs).is_ok()
    );

    let cdf = cdf.canonicalize().expect("failed to generate CDF file");

    println!("CDF file generated: {}", cdf.display());
}
