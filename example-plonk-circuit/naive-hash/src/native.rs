use dusk_plonk::prelude::*;

/// compute a naive hash operation that adds all scalars
pub fn hash(input: &[BlsScalar]) -> BlsScalar {
    input.iter().sum()
}
