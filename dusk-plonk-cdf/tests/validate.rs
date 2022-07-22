use std::cmp;

use dusk_plonk_cdf::*;
use dusk_plonk_debugger_utils::*;

#[test]
fn indexed_witness_validate_works() {
    let witnesses = 100;
    let constraints = 1000;

    let preamble = *Preamble::default()
        .with_witnesses(witnesses)
        .with_constraints(constraints);

    let mut generator = CDFGenerator::new(0x8437, preamble);

    let generated = generator.gen_indexed_witness();

    let id = generated.index();
    let origin = generated.origin();
    let value = generated.value();

    let ok_id = cmp::min(id, witnesses - 1);
    let wrong_id = cmp::max(id, witnesses);

    IndexedWitness::new(ok_id, *origin, *value)
        .validate(&preamble)
        .expect("consistent indexed witness should validate");

    IndexedWitness::new(wrong_id, *origin, *value)
        .validate(&preamble)
        .expect_err("inconsistent indexed witness shouldn't validate");

    let ok_constraint = constraints - 1;
    let wrong_constraint = constraints;

    IndexedWitness::new(ok_id, Some(ok_constraint), *value)
        .validate(&preamble)
        .expect("consistent indexed witness should validate");

    IndexedWitness::new(ok_id, Some(wrong_constraint), *value)
        .validate(&preamble)
        .expect_err("inconsistent indexed witness shouldn't validate");
}

#[test]
fn witness_validate_works() {
    let witnesses = 100;
    let constraints = 1000;

    let preamble = *Preamble::default()
        .with_witnesses(witnesses)
        .with_constraints(constraints);

    let mut generator = CDFGenerator::new(0x8437, preamble);

    let witness = generator.gen_witness();

    witness
        .source()
        .validate(&Default::default())
        .expect("failed to validate witness source");

    witness
        .validate(&Default::default())
        .expect("failed to validate witness");
}
