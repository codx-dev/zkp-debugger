use std::{fmt, io, iter};

use dusk_plonk_cdf::*;
use dusk_plonk_debugger_utils::*;

pub fn encode_decode_element<C, E, I>(mut elements: I)
where
    C: for<'a> From<&'a Config>,
    E: Element<Config = C> + fmt::Debug + PartialEq,
    I: Iterator<Item = E>,
{
    // TODO test all config variants
    let configs = vec![Config::default()];

    for config in configs {
        let config = C::from(&config);

        elements.by_ref().for_each(move |el| {
            let bytes = el.to_vec(&config);
            let el_p = E::try_from_buffer(&config, &bytes).expect("failed to decode");

            assert_eq!(el_p, el);

            let mut op = Some(el);
            let bytes = op.to_vec(&config);
            let op_p = <Option<E>>::try_from_buffer(&config, &bytes).expect("failed to decode");

            assert_eq!(op_p, op);
            assert_ne!(op_p, None);

            let el = op.take();

            let bytes = op.to_vec(&config);
            let op_p = <Option<E>>::try_from_buffer(&config, &bytes).expect("failed to decode");

            assert_eq!(op_p, None);
            assert_ne!(op_p, el);
        });
    }
}

#[test]
fn encode_language_primitives() {
    encode_decode_element(iter::once(true).chain(iter::once(false)));
    encode_decode_element(
        iter::once(u16::MIN)
            .chain(iter::once(u16::MAX))
            .chain(iter::once(u16::MAX >> 1)),
    );
    encode_decode_element(
        iter::once(u64::MIN)
            .chain(iter::once(u64::MAX))
            .chain(iter::once(u64::MAX >> 1)),
    );
    encode_decode_element(
        iter::once(usize::MIN)
            .chain(iter::once(usize::MAX))
            .chain(iter::once(usize::MAX >> 1)),
    );
    encode_decode_element(iter::once(()));
}

#[test]
fn encode_scalar_primitive() {
    encode_decode_element(iter::once(Scalar::from([0u8; Scalar::LEN])));
    encode_decode_element(iter::once(Scalar::from([0xfa; Scalar::LEN])));
}

#[test]
fn encode_fixed_text_primitive_and_source() {
    let text = String::from(
        "Anyone who can appease a man's conscience can take his freedom away from him.",
    );

    encode_decode_element(iter::once(FixedText::<2>::from(text.clone())));
    encode_decode_element(iter::once(FixedText::<3>::from(text.clone())));
    encode_decode_element(iter::once(FixedText::<4>::from(text.clone())));
    encode_decode_element(iter::once(FixedText::<5>::from(text.clone())));
    encode_decode_element(iter::once(FixedText::<6>::from(text.clone())));
    encode_decode_element(iter::once(FixedText::<7>::from(text.clone())));
    encode_decode_element(iter::once(FixedText::<8>::from(text.clone())));
    encode_decode_element(iter::once(FixedText::<9>::from(text.clone())));

    encode_decode_element(iter::once(Source::new(0, 0, text.clone().into())));
    encode_decode_element(iter::once(Source::new(u64::MAX, 0, text.clone().into())));
    encode_decode_element(iter::once(Source::new(0, u64::MAX, text.clone().into())));
    encode_decode_element(iter::once(Source::new(
        u64::MAX,
        u64::MAX,
        text.clone().into(),
    )));
}

#[test]
fn encode_preamble() {
    #[derive(Debug, PartialEq, Eq)]
    struct PreambleElement(Preamble);

    impl Element for PreambleElement {
        type Config = <Preamble as Element>::Config;

        fn zeroed() -> Self {
            Self(Default::default())
        }

        fn len(config: &Self::Config) -> usize {
            Preamble::len(config)
        }

        fn to_buffer(&self, config: &Self::Config, buf: &mut [u8]) {
            self.0.to_buffer(config, buf)
        }

        fn try_from_buffer_in_place(
            &mut self,
            config: &Self::Config,
            buf: &[u8],
        ) -> io::Result<()> {
            self.0.try_from_buffer_in_place(config, buf)
        }

        fn validate(&self, preamble: &Preamble) -> io::Result<()> {
            self.0.validate(preamble)
        }
    }

    encode_decode_element(iter::once(PreambleElement(
        *Preamble::new().with_witnesses(1).with_constraints(0),
    )));
    encode_decode_element(iter::once(PreambleElement(
        *Preamble::new()
            .with_witnesses(usize::MAX)
            .with_constraints(0),
    )));
    encode_decode_element(iter::once(PreambleElement(
        *Preamble::new()
            .with_witnesses(1)
            .with_constraints(usize::MAX),
    )));
    encode_decode_element(iter::once(PreambleElement(
        *Preamble::new()
            .with_witnesses(usize::MAX)
            .with_constraints(usize::MAX),
    )));
}

#[test]
fn encode_indexed_witness() {
    encode_decode_element(iter::once(IndexedWitness::new(
        0,
        None,
        [0xfa; Scalar::LEN].into(),
    )));
    encode_decode_element(iter::once(IndexedWitness::new(
        0,
        Some(0),
        [0xfa; Scalar::LEN].into(),
    )));
    encode_decode_element(iter::once(IndexedWitness::new(
        0,
        Some(u64::MAX),
        [0xfa; Scalar::LEN].into(),
    )));
    encode_decode_element(iter::once(IndexedWitness::new(
        u64::MAX,
        None,
        [0xfa; Scalar::LEN].into(),
    )));
    encode_decode_element(iter::once(IndexedWitness::new(
        u64::MAX,
        Some(0),
        [0xfa; Scalar::LEN].into(),
    )));
    encode_decode_element(iter::once(IndexedWitness::new(
        u64::MAX,
        Some(u64::MAX),
        [0xfa; Scalar::LEN].into(),
    )));
}

#[test]
fn encode_generated_witnesses() {
    let preamble = *Preamble::new().with_witnesses(100).with_constraints(10);
    let mut generator = CDFGenerator::new(0x8437, preamble);

    encode_decode_element((0..100).map(|_| generator.gen_witness()));
}

#[test]
fn encode_generated_constraints() {
    let preamble = *Preamble::new().with_witnesses(100).with_constraints(10);
    let mut generator = CDFGenerator::new(0x8437, preamble);

    encode_decode_element((0..100).map(|_| generator.gen_constraint()));
}
