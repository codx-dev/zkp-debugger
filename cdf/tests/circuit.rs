use std::{fs, iter};

use dusk_cdf::*;
use dusk_zkp_debugger_utils::*;
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

        if witnesses.len() > 1 {
            assert_ne!(witnesses, shuffled_w);
        }

        if constraints.len() > 1 {
            assert_ne!(constraints, shuffled_c);
        }

        let config = Config::default();
        let mut encoder =
            Encoder::init_cursor(config, shuffled_w.into_iter(), shuffled_c.into_iter());

        encoder
            .write_all()
            .expect("encoder failed to receive shuffled input");

        let mut cursor = encoder.into_inner();
        cursor.set_position(0);
        let mut cdf = CircuitDescription::from_reader(cursor).expect("failed to open cdf");

        witnesses.iter().for_each(|w| {
            let id = w.id();
            let w_p = cdf
                .fetch_witness(id as usize)
                .expect("failed to fetch witnesss");

            assert_eq!(w, &w_p);
        });

        constraints.iter().for_each(|c| {
            let id = c.id();
            let c_p = cdf
                .fetch_constraint(id as usize)
                .expect("failed to fetch constraint");

            assert_eq!(c, &c_p);
        });
    }
}

#[test]
fn single_witness_circuit_is_valid() {
    let preamble = *Preamble::new().with_witnesses(1).with_constraints(0);
    let mut generator = CDFGenerator::new(0x384, preamble);

    let id = 0;
    let value = generator.gen_scalar();
    let source = generator.gen_source();

    let witness = Witness::new(id, value, source);

    let config = Config::default();
    let mut encoder =
        Encoder::init_cursor(config, iter::once(witness), iter::empty::<Constraint>());

    encoder
        .write_all()
        .expect("encoder failed to receive shuffled input");

    let mut cursor = encoder.into_inner();

    cursor.set_position(0);

    let cdf = CircuitDescription::from_reader(cursor).expect("failed to open cdf");
    let preamble = cdf.preamble();

    assert_eq!(preamble.witnesses, 1);
    assert_eq!(preamble.constraints, 0);
}

#[test]
fn witness_must_start_at_zero() {
    let preamble = *Preamble::new().with_witnesses(1).with_constraints(0);
    let mut generator = CDFGenerator::new(0x384, preamble);

    let id = 1;
    let value = generator.gen_scalar();
    let source = generator.gen_source();

    let witness = Witness::new(id, value, source);

    let config = Config::default();

    Encoder::init_cursor(config, iter::once(witness), iter::empty::<Constraint>())
        .write_all()
        .expect_err("invalid first witness shouldn't encode");
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

    let config = Config::default();

    // Sanity check
    Encoder::init_cursor(
        config,
        iter::once(witness.clone()),
        iter::once(constraint.clone()),
    )
    .write_all()
    .expect("failed to validate circuit");

    let id = 1;
    let source = generator.gen_source();

    let constraint = Constraint::new(id, polynomial, source);

    Encoder::init_cursor(config, iter::once(witness), iter::once(constraint))
        .write_all()
        .expect_err("invalid first witness shouldn't encode");
}

#[test]
fn circuit_data_seek_works_for_witness_and_constraints() {
    let config = Config::default();
    let preambles = vec![
        *Preamble::new().with_witnesses(1).with_constraints(0),
        *Preamble::new().with_witnesses(1).with_constraints(10),
        *Preamble::new().with_witnesses(10).with_constraints(0),
        *Preamble::new().with_witnesses(10).with_constraints(100),
    ];

    for preamble in preambles {
        let mut generator = CDFGenerator::new(0x384, preamble);
        let (witnesses, constraints) = generator.gen_structurally_sound_circuit();

        let mut encoder = Encoder::init_cursor(config, witnesses.iter(), constraints.iter());

        encoder
            .write_all()
            .expect("encoder failed to receive shuffled input");

        let mut cursor = encoder.into_inner();

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

#[test]
fn file_write_works() {
    let dir = tempfile::tempdir().expect("failed to create temporary dir");
    let file = dir.path().join("file-write-works.cdf");

    let preamble = *Preamble::new().with_witnesses(10).with_constraints(100);

    let mut generator = CDFGenerator::new(0x384, preamble);
    let (witnesses, constraints) = generator.gen_structurally_sound_circuit();

    Encoder::init_file(preamble.config, witnesses.iter(), constraints.iter(), &file)
        .expect("failed to init file")
        .write_all()
        .expect("failed to write file");

    let mut cdf = CircuitDescription::open_read(file).expect("failed to open generated cdf file");

    witnesses.iter().for_each(|w| {
        let id = w.id();
        let w_p = cdf
            .fetch_witness(id as usize)
            .expect("failed to fetch witness");

        assert_eq!(w, &w_p);
    });

    constraints.iter().for_each(|c| {
        let id = c.id();
        let c_p = cdf
            .fetch_constraint(id as usize)
            .expect("failed to fetch constraint");

        assert_eq!(c, &c_p);
    });

    fs::remove_dir_all(dir).ok();
}
