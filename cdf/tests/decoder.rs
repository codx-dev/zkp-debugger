use std::path::PathBuf;

use dusk_cdf::*;

#[test]
fn decoder_works() {
    let asset = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("failed to find root workspace dir")
        .join("assets")
        .join("test.cdf")
        .canonicalize()
        .expect("failed to find CDF test asset");

    let mut cdf =
        CircuitDescription::open(asset).expect("failed to read test asset");

    // assert data was fetched
    assert_ne!(cdf.preamble().witnesses, 0);
    assert_ne!(cdf.preamble().constraints, 0);

    // attempt to decode all witnesses
    for idx in 0..cdf.preamble().witnesses {
        cdf.fetch_witness(idx).expect("failed to read witness");
    }

    // attempt to decode all constraints
    for idx in 0..cdf.preamble().constraints {
        cdf.fetch_constraint(idx)
            .expect("failed to read constraint");
    }

    // assert attempts to fetch invalid witnesses won't panic
    cdf.fetch_witness(cdf.preamble().witnesses)
        .expect_err("witness doesn't exist in the set");

    // assert attempts to fetch invalid constraints won't panic
    cdf.fetch_constraint(cdf.preamble().constraints)
        .expect_err("constraint doesn't exist in the set");
}
