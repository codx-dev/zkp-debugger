use std::{io, iter};

use dusk_plonk_cdf::*;
use dusk_plonk_debugger_utils::*;
use rand::prelude::*;

#[test]
fn shuffled_circuit_is_sound_after_validation() {
    let preambles = vec![
        *Preamble::new().with_witnesses(1).with_constraints(0),
        *Preamble::new().with_witnesses(1).with_constraints(1),
        *Preamble::new().with_witnesses(1).with_constraints(10),
        *Preamble::new().with_witnesses(100).with_constraints(1000),
    ];

    for preamble in preambles {
        let mut generator = CDFGenerator::new(0x384, preamble);
        let (witnesses, constraints) = generator.gen_structurally_sound_circuit();

        let mut shuffled_w = witnesses.clone();
        let mut shuffled_c = constraints.clone();

        shuffled_w.shuffle(&mut generator);
        shuffled_c.shuffle(&mut generator);

        if witnesses.len() != shuffled_w.len() {
            assert_ne!(witnesses, shuffled_w);
        }

        if constraints.len() != shuffled_c.len() {
            assert_ne!(constraints, shuffled_c);
        }

        let (p, w, c) = CircuitDescriptionUnit::into_valid_cdf(
            Preamble::new(),
            shuffled_w.into_iter(),
            shuffled_c.into_iter(),
        )
        .expect("failed to validate circuit");

        let w: Vec<Witness> = w.collect();
        let c: Vec<Constraint> = c.collect();

        w.iter()
            .enumerate()
            .for_each(|(i, w)| assert_eq!(i as u64, w.id()));

        c.iter()
            .enumerate()
            .for_each(|(i, c)| assert_eq!(i as u64, c.id()));

        assert_eq!(preamble, p);
        assert_eq!(witnesses, w);
        assert_eq!(constraints, c);
    }
}

#[test]
#[should_panic]
fn witness_count_cant_be_zero() {
    Preamble::new().with_witnesses(0).with_constraints(100);
}

#[test]
fn single_witness_circuit_is_valid() {
    let preamble = *Preamble::new().with_witnesses(1).with_constraints(0);
    let mut generator = CDFGenerator::new(0x384, preamble);

    let id = 0;
    let value = generator.gen_scalar();
    let source = generator.gen_source();

    let witness = Witness::new(id, value, source);

    let (p, _, _) = CircuitDescriptionUnit::into_valid_cdf(
        Default::default(),
        iter::once(witness),
        iter::empty(),
    )
    .expect("failed to validate circuit");

    assert_eq!(p.witnesses, 1);
    assert_eq!(p.constraints, 0);
}

#[test]
fn witness_must_start_at_zero() {
    let preamble = *Preamble::new().with_witnesses(1).with_constraints(0);
    let mut generator = CDFGenerator::new(0x384, preamble);

    let id = 1;
    let value = generator.gen_scalar();
    let source = generator.gen_source();

    let witness = Witness::new(id, value, source);

    let result = CircuitDescriptionUnit::into_valid_cdf(
        Default::default(),
        iter::once(witness),
        iter::empty(),
    );

    assert!(result.is_err());
}

#[test]
fn constraint_must_start_at_zero() {
    let preamble = *Preamble::new().with_witnesses(1).with_constraints(1);
    let mut generator = CDFGenerator::new(0x384, preamble);

    let (witnesses, constraints) = generator.gen_structurally_sound_circuit();

    let witness = witnesses[0].clone();
    let polynomial = constraints[0].polynomial().clone();

    let id = 0;
    let source = generator.gen_source();

    let constraint = Constraint::new(id, polynomial, source);

    // Sanity check
    let (p, _, _) = CircuitDescriptionUnit::into_valid_cdf(
        Default::default(),
        iter::once(witness.clone()),
        iter::once(constraint.clone()),
    )
    .expect("failed to validate circuit");

    assert_eq!(preamble, p);

    let id = 1;
    let source = generator.gen_source();

    let constraint = Constraint::new(id, polynomial, source);

    let result = CircuitDescriptionUnit::into_valid_cdf(
        Default::default(),
        iter::once(witness),
        iter::once(constraint),
    );

    assert!(result.is_err());
}

#[test]
fn circuit_data_seek_works_for_witness_and_constraints() {
    let preambles = vec![
        *Preamble::new().with_witnesses(1).with_constraints(0),
        *Preamble::new().with_witnesses(1).with_constraints(10),
        *Preamble::new().with_witnesses(10).with_constraints(0),
        *Preamble::new().with_witnesses(10).with_constraints(100),
    ];

    for preamble in preambles {
        let mut generator = CDFGenerator::new(0x384, preamble);
        let (witnesses, constraints) = generator.gen_structurally_sound_circuit();

        let mut cursor = io::Cursor::new(Vec::new());

        CircuitDescriptionUnit::write_all(
            &mut cursor,
            Default::default(),
            witnesses.clone().into_iter(),
            constraints.clone().into_iter(),
        )
        .expect("failed to serialize circuit");

        // Reset the cursor to open the circuit
        cursor.set_position(0);

        let mut circuit =
            CircuitDescription::from_reader(&mut cursor).expect("failed to open circuit");

        assert_eq!(&preamble, circuit.preamble());

        witnesses
            .iter()
            .enumerate()
            .rev()
            .for_each(|(idx, witness)| {
                let w = circuit.fetch_witness(idx).expect("failed to fetch witness");

                assert_eq!(witness, &w);
            });

        constraints
            .iter()
            .enumerate()
            .rev()
            .for_each(|(idx, constraint)| {
                let c = circuit
                    .fetch_constraint(idx)
                    .expect("failed to fetch constraint");

                assert_eq!(constraint, &c);
            });

        // check if won't panic on invalid idx
        let result = circuit.fetch_witness(witnesses.len());

        assert!(result.is_err());

        // check if won't panic on invalid idx
        let result = circuit.fetch_constraint(constraints.len());

        assert!(result.is_err());
    }
}
