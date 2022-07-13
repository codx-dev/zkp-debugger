use dusk_plonk::prelude::*;

pub fn quad(composer: &mut TurboComposer, a: Witness, b: Witness) -> Witness {
    let constraint = Constraint::new().mult(1).a(a).b(a);
    let x = composer.gate_mul(constraint);

    let constraint = Constraint::new().mult(2).a(a).b(b).constant(5);
    let y = composer.gate_mul(constraint);

    let constraint = Constraint::new().left(1).a(x).right(1).b(y);

    composer.gate_add(constraint)
}

pub fn sub(composer: &mut TurboComposer, a: Witness, b: Witness) -> Witness {
    let constraint = Constraint::new().left(1).a(a).right(-BlsScalar::one()).b(b);

    composer.gate_add(constraint)
}
