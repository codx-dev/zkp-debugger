use std::marker::PhantomData;
use std::{fmt, io, iter};

use dusk_cdf::*;
use dusk_zkp_debugger_utils::*;

pub fn encode_decode_element<C, E, I, S>(mut elements: I, mut ctx: Context<S>)
where
    C: for<'a> From<&'a Config>,
    E: Element<Config = C> + fmt::Debug + PartialEq,
    I: Iterator<Item = E>,
    S: io::Read + io::Seek,
{
    // TODO test all config variants
    let configs = vec![Config::default()];

    for config in configs {
        let config = C::from(&config);

        elements.by_ref().for_each(|el| {
            let ctx_encode = &mut Context::unit();

            let bytes = el.to_vec(&config, ctx_encode.with_source_cache_id(0));

            if !bytes.is_empty() {
                E::try_from_buffer(&config, &mut ctx, &[])
                    .expect_err("decode from empty buffer shouldn't panic");
            }

            let el_p = E::try_from_buffer(&config, &mut ctx, &bytes).expect("failed to decode");

            assert_eq!(el_p, el);

            let mut op = Some(el);
            let bytes = op.to_vec(&config, ctx_encode.with_source_cache_id(0));
            let op_p =
                <Option<E>>::try_from_buffer(&config, &mut ctx, &bytes).expect("failed to decode");

            assert_eq!(op_p, op);
            assert_ne!(op_p, None);

            let el = op.take();

            let bytes = op.to_vec(&config, ctx_encode.with_source_cache_id(0));
            let op_p =
                <Option<E>>::try_from_buffer(&config, &mut ctx, &bytes).expect("failed to decode");

            assert_eq!(op_p, None);
            assert_ne!(op_p, el);
        });
    }
}

#[test]
fn encode_language_primitives() {
    let preamble = *Preamble::new().with_witnesses(1).with_constraints(0);
    let mut generator = CDFGenerator::new(0x384, preamble);
    let mut cdf = generator.gen_cdf();

    encode_decode_element(
        vec![
            *Preamble::new().with_witnesses(1).with_constraints(0),
            *Preamble::new().with_witnesses(1).with_constraints(1),
            *Preamble::new().with_witnesses(1).with_constraints(10),
            *Preamble::new().with_witnesses(10).with_constraints(0),
            *Preamble::new().with_witnesses(10).with_constraints(1),
            *Preamble::new().with_witnesses(10).with_constraints(10),
        ]
        .into_iter(),
        cdf.context(),
    );

    let phantom: PhantomData<u64> = PhantomData;
    encode_decode_element(iter::once(phantom), cdf.context());
    encode_decode_element(iter::once(()), cdf.context());

    encode_decode_element(iter::once(true).chain(iter::once(false)), cdf.context());
    encode_decode_element(
        iter::once(u16::MIN)
            .chain(iter::once(u16::MAX))
            .chain(iter::once(u16::MAX >> 1)),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(u64::MIN)
            .chain(iter::once(u64::MAX))
            .chain(iter::once(u64::MAX >> 1)),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(usize::MIN)
            .chain(iter::once(usize::MAX))
            .chain(iter::once(usize::MAX >> 1)),
        cdf.context(),
    );
    encode_decode_element(iter::once(()), cdf.context());
}

#[test]
fn encode_scalar_primitive() {
    let preamble = *Preamble::new().with_witnesses(1).with_constraints(0);
    let mut generator = CDFGenerator::new(0x384, preamble);
    let mut cdf = generator.gen_cdf();

    encode_decode_element(iter::once(Scalar::from([0u8; Scalar::LEN])), cdf.context());
    encode_decode_element(iter::once(Scalar::from([0xfa; Scalar::LEN])), cdf.context());
}

#[test]
fn encode_fixed_text_primitive_and_source() {
    let preamble = *Preamble::new().with_witnesses(1).with_constraints(0);
    let mut generator = CDFGenerator::new(0x384, preamble);
    let mut cdf = generator.gen_cdf();
    let source = cdf
        .fetch_source(0)
        .expect("failed to fetch source from generated witness");

    let text = String::from(
        "Anyone who can appease a man's conscience can take his freedom away from him.",
    );

    encode_decode_element(
        iter::once(FixedText::<2>::from(text.clone())),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(FixedText::<3>::from(text.clone())),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(FixedText::<4>::from(text.clone())),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(FixedText::<5>::from(text.clone())),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(FixedText::<6>::from(text.clone())),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(FixedText::<7>::from(text.clone())),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(FixedText::<8>::from(text.clone())),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(FixedText::<9>::from(text.clone())),
        cdf.context(),
    );

    encode_decode_element(iter::once(Source::new(0, 0, source.clone())), cdf.context());

    encode_decode_element(
        iter::once(Source::new(u64::MAX, 0, source.clone())),
        cdf.context(),
    );

    encode_decode_element(
        iter::once(Source::new(0, u64::MAX, source.clone())),
        cdf.context(),
    );

    encode_decode_element(
        iter::once(Source::new(u64::MAX, u64::MAX, source.clone())),
        cdf.context(),
    );
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

        fn to_buffer(&self, config: &Self::Config, context: &mut Context<()>, buf: &mut [u8]) {
            self.0.to_buffer(config, context, buf)
        }

        fn try_from_buffer_in_place<S>(
            &mut self,
            config: &Self::Config,
            context: &mut Context<S>,
            buf: &[u8],
        ) -> io::Result<()>
        where
            S: io::Read + io::Seek,
        {
            self.0.try_from_buffer_in_place(config, context, buf)
        }

        fn validate(&self, preamble: &Preamble) -> io::Result<()> {
            self.0.validate(preamble)
        }
    }

    PreambleElement(Default::default())
        .validate(&Default::default())
        .expect("default preamble should validate");

    let preamble = *Preamble::new().with_witnesses(1).with_constraints(0);
    let mut generator = CDFGenerator::new(0x384, preamble);
    let mut cdf = generator.gen_cdf();

    encode_decode_element(
        iter::once(PreambleElement(
            *Preamble::new().with_witnesses(1).with_constraints(0),
        )),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(PreambleElement(
            *Preamble::new()
                .with_witnesses(usize::MAX)
                .with_constraints(0),
        )),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(PreambleElement(
            *Preamble::new()
                .with_witnesses(1)
                .with_constraints(usize::MAX),
        )),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(PreambleElement(
            *Preamble::new()
                .with_witnesses(usize::MAX)
                .with_constraints(usize::MAX),
        )),
        cdf.context(),
    );
}

#[test]
fn encode_indexed_witness() {
    let preamble = *Preamble::new().with_witnesses(1).with_constraints(0);
    let mut generator = CDFGenerator::new(0x384, preamble);
    let mut cdf = generator.gen_cdf();

    encode_decode_element(
        iter::once(IndexedWitness::new(0, None, [0xfa; Scalar::LEN].into())),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(IndexedWitness::new(0, Some(0), [0xfa; Scalar::LEN].into())),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(IndexedWitness::new(
            0,
            Some(usize::MAX),
            [0xfa; Scalar::LEN].into(),
        )),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(IndexedWitness::new(
            usize::MAX,
            None,
            [0xfa; Scalar::LEN].into(),
        )),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(IndexedWitness::new(
            usize::MAX,
            Some(0),
            [0xfa; Scalar::LEN].into(),
        )),
        cdf.context(),
    );
    encode_decode_element(
        iter::once(IndexedWitness::new(
            usize::MAX,
            Some(usize::MAX),
            [0xfa; Scalar::LEN].into(),
        )),
        cdf.context(),
    );
}

#[test]
fn encode_polynomial() {
    let preamble = *Preamble::new().with_witnesses(1).with_constraints(0);
    let mut generator = CDFGenerator::new(0x384, preamble);
    let mut cdf = generator.gen_cdf();

    encode_decode_element((0..100).map(|_| generator.gen_polynomial()), cdf.context());
}

#[test]
fn invalid_utf8_wont_panic() {
    let preamble = *Preamble::new().with_witnesses(100).with_constraints(10);
    let mut generator = CDFGenerator::new(0x8437, preamble);
    let mut cdf = generator.gen_cdf();

    let config = preamble.config;

    let invalid_utf8 = vec![0, 159, 146, 150];
    let mut buffer = (invalid_utf8.len() as u16).to_le_bytes().to_vec();

    buffer.extend(&invalid_utf8);

    String::from_utf8(invalid_utf8).expect_err("invalid char shouldn't generate string");

    FixedText::<1>::try_from_buffer(&config, &mut cdf.context(), &buffer)
        .expect_err("invalid char shouldn't generate fixed text");
}

#[test]
fn try_from_buffer_in_place_works_with_some() {
    let preamble = *Preamble::new().with_witnesses(100).with_constraints(10);
    let mut generator = CDFGenerator::new(0x8437, preamble);
    let mut cdf = generator.gen_cdf();

    let val = 39802u64;
    let ctx = &mut Context::unit();

    let bytes = Some(val).to_vec(&AtomicConfig, ctx);

    let mut some = Some(1);

    some.try_from_buffer_in_place(&AtomicConfig, &mut cdf.context(), &bytes)
        .expect("failed to restore option");

    let val_p = some.expect("failed to fetch val");

    assert_eq!(val, val_p);
}
