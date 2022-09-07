use dusk_plonk::prelude::*;

/// compute a naive signature that adds `key` to the result of the naive hash of the inputs
pub fn sign(key: &BlsScalar, input: &[BlsScalar]) -> BlsScalar {
    naive_hash::native::hash(input) + key
}

pub fn verify(key: &BlsScalar, signature: &BlsScalar, input: &[BlsScalar]) -> bool {
    let message = naive_hash::native::hash(input);
    let derived = signature - message;

    key == &derived
}
