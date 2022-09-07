use dusk_plonk::prelude::*;

/// compute a naive hash operation that adds all witnesses
pub fn hash<C>(composer: &mut C, input: &[Witness]) -> Witness
where
    C: Composer,
{
    input.iter().fold(C::ZERO, |acc, w| {
        let constraint = Constraint::new().left(1).a(acc).right(1).b(*w);

        composer.gate_add(constraint)
    })
}
