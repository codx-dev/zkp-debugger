mod gadgets;

use dusk_plonk::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DummyCircuit {
    pub a: BlsScalar,
    pub b: BlsScalar,
    pub c: BlsScalar,
    pub d: BlsScalar,
}

impl Circuit for DummyCircuit {
    const CIRCUIT_ID: [u8; 32] = [0xfa; 32];

    fn gadget(&mut self, composer: &mut TurboComposer) -> Result<(), Error> {
        let a = composer.append_witness(self.a);
        let b = composer.append_witness(self.b);
        let c = composer.append_witness(self.c);
        let d = composer.append_witness(self.d);

        let x = gadgets::quad(composer, a, b);
        let y = gadgets::sub(composer, a, x);

        composer.assert_equal(c, x);
        composer.assert_equal(d, y);

        Ok(())
    }

    fn public_inputs(&self) -> Vec<PublicInputValue> {
        vec![]
    }

    fn padded_gates(&self) -> usize {
        1 << 8
    }

}
