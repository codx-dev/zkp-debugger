use std::{fmt, iter};

use dusk_plonk_cdf::*;
use dusk_plonk_debugger_utils::*;

pub fn encode_decode_element<E, I>(mut elements: I)
where
    E: Element + fmt::Debug + PartialEq,
    I: Iterator<Item = E>,
{
    let preambles = vec![
        Preamble::new(1, 0),
        Preamble::new(1, 1),
        Preamble::new(1, 10),
        Preamble::new(10, 0),
        Preamble::new(10, 1),
        Preamble::new(10, 10),
        Preamble::new(10, 100),
    ];

    for preamble in preambles {
        elements.by_ref().for_each(move |el| {
            let bytes = el.to_vec(&preamble);
            let el_p = E::try_from_buffer(&preamble, &bytes).expect("failed to decode");

            assert_eq!(el_p, el);

            let mut op = Some(el);
            let bytes = op.to_vec(&preamble);
            let op_p = <Option<E>>::try_from_buffer(&preamble, &bytes).expect("failed to decode");

            assert_eq!(op_p, op);
            assert_ne!(op_p, None);

            let el = op.take();

            let bytes = op.to_vec(&preamble);
            let op_p = <Option<E>>::try_from_buffer(&preamble, &bytes).expect("failed to decode");

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
        fn zeroed() -> Self {
            Self(Preamble::ZEROED)
        }

        fn len(_preamble: &Preamble) -> usize {
            Preamble::LEN
        }

        fn to_buffer(&self, _preamble: &Preamble, buf: &mut [u8]) {
            self.0.to_buffer(buf)
        }

        fn try_from_buffer_in_place(
            &mut self,
            _preamble: &Preamble,
            buf: &[u8],
        ) -> std::io::Result<()> {
            self.0.try_from_buffer_in_place(buf)
        }

        fn validate(&self, _preamble: &Preamble) -> std::io::Result<()> {
            Ok(())
        }
    }

    encode_decode_element(iter::once(PreambleElement(Preamble::new(1, 0))));
    encode_decode_element(iter::once(PreambleElement(Preamble::new(u64::MAX, 0))));
    encode_decode_element(iter::once(PreambleElement(Preamble::new(1, u64::MAX))));
    encode_decode_element(iter::once(PreambleElement(Preamble::new(
        u64::MAX,
        u64::MAX,
    ))));
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
    let preamble = Preamble::new(100, 10);
    let mut generator = CDFGenerator::new(0x8437, preamble);

    encode_decode_element((0..100).map(|_| generator.gen_witness()));
}

#[test]
fn encode_generated_constraints() {
    let preamble = Preamble::new(100, 10);
    let mut generator = CDFGenerator::new(0x8437, preamble);

    encode_decode_element((0..100).map(|_| generator.gen_constraint()));
}
