use dusk_plonk::prelude::*;

/// compute a naive signature that adds `key` to the result of the naive hash of the inputs
pub fn sign<C>(composer: &mut C, key: Witness, input: &[Witness]) -> Witness
where
    C: Composer,
{
    let message = naive_hash::gadget::hash(composer, input);
    let constraint = Constraint::new().left(1).a(key).right(1).b(message);

    composer.gate_add(constraint)
}

pub fn verify<C>(composer: &mut C, key: Witness, signature: Witness, input: &[Witness])
where
    C: Composer,
{
    let message = naive_hash::gadget::hash(composer, input);

    let constraint = Constraint::new()
        .left(-BlsScalar::one())
        .a(message)
        .right(1)
        .b(signature);

    let derived = composer.gate_add(constraint);

    composer.assert_equal(key, derived);
}
