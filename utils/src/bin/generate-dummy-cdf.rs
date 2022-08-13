use std::path::PathBuf;

use dusk_bls12_381::BlsScalar;
use dusk_bytes::Serializable;
use dusk_cdf::*;

fn main() {
    let output = std::env::var("CDF_OUTPUT").unwrap();
    let output = PathBuf::from(output);

    let lib = std::env::var("CDF_LIB").unwrap();
    let lib = PathBuf::from(lib)
        .to_str()
        .expect("failed to get str of path")
        .to_string();

    let gadgets = std::env::var("CDF_GADGETS").unwrap();
    let gadgets = PathBuf::from(gadgets)
        .to_str()
        .expect("failed to get str of path")
        .to_string();

    let lib: FixedText<{ Source::PATH_LEN }> = FixedText::from(lib);
    let gadgets: FixedText<{ Source::PATH_LEN }> = FixedText::from(gadgets);

    let zero = BlsScalar::from(0);
    let zero = Scalar::from(zero.to_bytes());
    let zero_w = IndexedWitness::new(0, None, zero);

    let mut witnesses = vec![];
    let mut constraints = vec![];

    let mut witness_id = 0;
    let mut constraint_id = 0;

    let source = Source::new(14, 0, lib.clone());
    let witness = Witness::new(witness_id, zero, source);
    witness_id += 1;

    witnesses.push(witness);

    let val = BlsScalar::from(2);
    let val = Scalar::from(val.to_bytes());
    let source = Source::new(17, 0, lib.clone());
    let witness = Witness::new(witness_id, val, source);
    witness_id += 1;

    witnesses.push(witness);

    let val = BlsScalar::from(3);
    let val = Scalar::from(val.to_bytes());
    let source = Source::new(18, 0, lib.clone());
    let witness = Witness::new(witness_id, val, source);
    witness_id += 1;

    witnesses.push(witness);

    let val = BlsScalar::from(21);
    let val = Scalar::from(val.to_bytes());
    let source = Source::new(19, 0, lib.clone());
    let witness = Witness::new(witness_id, val, source);
    witness_id += 1;

    witnesses.push(witness);

    let val = BlsScalar::from(2) - BlsScalar::from(21);
    let val = Scalar::from(val.to_bytes());
    let source = Source::new(20, 0, lib.clone());
    let witness = Witness::new(witness_id, val, source);
    witness_id += 1;

    witnesses.push(witness);

    let val = BlsScalar::from(4);
    let val = Scalar::from(val.to_bytes());
    let source = Source::new(5, 0, gadgets.clone());
    let witness = Witness::new(witness_id, val, source);
    witness_id += 1;

    witnesses.push(witness);

    let qm = BlsScalar::from(1);
    let qm = Scalar::from(qm.to_bytes());
    let qo = -BlsScalar::from(1);
    let qo = Scalar::from(qo.to_bytes());
    let a = IndexedWitness::new(1, None, witnesses[1].value().clone());
    let b = IndexedWitness::new(1, None, witnesses[1].value().clone());
    let o = IndexedWitness::new(5, None, witnesses[5].value().clone());
    let poly = Polynomial::new(
        qm, zero, zero, zero, zero, qo, zero, zero, zero, zero, a, b, zero_w, o, true,
    );
    let source = Source::new(5, 0, gadgets.clone());
    let constraint = Constraint::new(constraint_id, poly, source);
    constraint_id += 1;

    constraints.push(constraint);

    let val = BlsScalar::from(17);
    let val = Scalar::from(val.to_bytes());
    let source = Source::new(8, 0, gadgets.clone());
    let witness = Witness::new(witness_id, val, source);
    witness_id += 1;

    witnesses.push(witness);

    let qm = BlsScalar::from(2);
    let qm = Scalar::from(qm.to_bytes());
    let qo = -BlsScalar::from(1);
    let qo = Scalar::from(qo.to_bytes());
    let qc = BlsScalar::from(5);
    let qc = Scalar::from(qc.to_bytes());
    let a = IndexedWitness::new(1, None, witnesses[1].value().clone());
    let b = IndexedWitness::new(2, None, witnesses[2].value().clone());
    let o = IndexedWitness::new(6, None, witnesses[6].value().clone());
    let poly = Polynomial::new(
        qm, zero, zero, zero, qc, qo, zero, zero, zero, zero, a, b, zero_w, o, true,
    );
    let source = Source::new(8, 0, gadgets.clone());
    let constraint = Constraint::new(constraint_id, poly, source);
    constraint_id += 1;

    constraints.push(constraint);

    let val = BlsScalar::from(21);
    let val = Scalar::from(val.to_bytes());
    let source = Source::new(12, 0, gadgets.clone());
    let witness = Witness::new(witness_id, val, source);
    witness_id += 1;

    witnesses.push(witness);

    let ql = BlsScalar::from(1);
    let ql = Scalar::from(ql.to_bytes());
    let qr = BlsScalar::from(1);
    let qr = Scalar::from(qr.to_bytes());
    let qo = -BlsScalar::from(1);
    let qo = Scalar::from(qo.to_bytes());
    let a = IndexedWitness::new(5, None, witnesses[5].value().clone());
    let b = IndexedWitness::new(6, None, witnesses[6].value().clone());
    let o = IndexedWitness::new(7, None, witnesses[7].value().clone());
    let poly = Polynomial::new(
        zero, ql, qr, zero, zero, qo, zero, zero, zero, zero, a, b, zero_w, o, true,
    );
    let source = Source::new(12, 0, gadgets.clone());
    let constraint = Constraint::new(constraint_id, poly, source);
    constraint_id += 1;

    constraints.push(constraint);

    let val = BlsScalar::from(2) - BlsScalar::from(21);
    let val = Scalar::from(val.to_bytes());
    let source = Source::new(18, 0, gadgets.clone());
    let witness = Witness::new(witness_id, val, source);

    witnesses.push(witness);

    let ql = BlsScalar::from(1);
    let ql = Scalar::from(ql.to_bytes());
    let qr = -BlsScalar::one();
    let qr = Scalar::from(qr.to_bytes());
    let qo = -BlsScalar::from(1);
    let qo = Scalar::from(qo.to_bytes());
    let a = IndexedWitness::new(1, None, witnesses[1].value().clone());
    let b = IndexedWitness::new(7, None, witnesses[7].value().clone());
    let o = IndexedWitness::new(8, None, witnesses[8].value().clone());
    let poly = Polynomial::new(
        zero, ql, qr, zero, zero, qo, zero, zero, zero, zero, a, b, zero_w, o, true,
    );
    let source = Source::new(18, 0, gadgets.clone());
    let constraint = Constraint::new(constraint_id, poly, source);
    constraint_id += 1;

    constraints.push(constraint);

    let ql = BlsScalar::from(1);
    let ql = Scalar::from(ql.to_bytes());
    let qr = -BlsScalar::one();
    let qr = Scalar::from(qr.to_bytes());
    let a = IndexedWitness::new(3, None, witnesses[3].value().clone());
    let b = IndexedWitness::new(7, None, witnesses[7].value().clone());
    let poly = Polynomial::new(
        zero, ql, qr, zero, zero, zero, zero, zero, zero, zero, a, b, zero_w, zero_w, true,
    );
    let source = Source::new(25, 0, lib.clone());
    let constraint = Constraint::new(constraint_id, poly, source);
    constraint_id += 1;

    constraints.push(constraint);

    let ql = BlsScalar::from(1);
    let ql = Scalar::from(ql.to_bytes());
    let qr = -BlsScalar::one();
    let qr = Scalar::from(qr.to_bytes());
    let a = IndexedWitness::new(4, None, witnesses[4].value().clone());
    let b = IndexedWitness::new(8, None, witnesses[8].value().clone());
    let poly = Polynomial::new(
        zero, ql, qr, zero, zero, zero, zero, zero, zero, zero, a, b, zero_w, zero_w, true,
    );
    let source = Source::new(26, 0, lib.clone());
    let constraint = Constraint::new(constraint_id, poly, source);

    constraints.push(constraint);

    let config = Config::DEFAULT;

    Encoder::init_file(
        config,
        witnesses.into_iter(),
        constraints.into_iter(),
        output,
    )
    .and_then(|mut f| f.write_all())
    .expect("failed to write output OK file");
}
