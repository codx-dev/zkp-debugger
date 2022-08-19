use std::fmt;

use crate::source::EncodedSource;
use crate::*;
use quickcheck::{quickcheck, Arbitrary, Gen, TestResult};

impl Arbitrary for Scalar {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut bytes = [0u8; 32];

        bytes.iter_mut().for_each(|b| *b = u8::arbitrary(g));

        bytes.into()
    }
}

impl Arbitrary for Config {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            zeroed_scalar_values: bool::arbitrary(g),
        }
    }
}

impl Arbitrary for WiredWitnesses {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            a: usize::arbitrary(g),
            b: usize::arbitrary(g),
            d: usize::arbitrary(g),
            o: usize::arbitrary(g),
        }
    }
}

impl Arbitrary for Selectors {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            qm: Scalar::arbitrary(g),
            ql: Scalar::arbitrary(g),
            qr: Scalar::arbitrary(g),
            qd: Scalar::arbitrary(g),
            qc: Scalar::arbitrary(g),
            qo: Scalar::arbitrary(g),
            pi: Scalar::arbitrary(g),
            qarith: Scalar::arbitrary(g),
            qlogic: Scalar::arbitrary(g),
            qrange: Scalar::arbitrary(g),
            qgroup_variable: Scalar::arbitrary(g),
            qfixed_add: Scalar::arbitrary(g),
        }
    }
}

impl Arbitrary for Preamble {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            witnesses: usize::arbitrary(g).min(1),
            constraints: usize::arbitrary(g),
            config: Config::arbitrary(g),
        }
    }
}

impl Arbitrary for Polynomial {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            selectors: Selectors::arbitrary(g),
            witnesses: WiredWitnesses::arbitrary(g),
            evaluation: bool::arbitrary(g),
        }
    }
}

impl Arbitrary for EncodedSource {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            line: u64::arbitrary(g),
            col: u64::arbitrary(g),
            contents_index: usize::arbitrary(g),
        }
    }
}

#[test]
fn elements() {
    fn prop<T>(len: u8, data: T) -> TestResult
    where
        T: Arbitrary + EncodableElement + DecodableElement + PartialEq + fmt::Debug,
    {
        let len = len as usize;
        let min = T::len(&Default::default());

        if len < min {
            return TestResult::discard();
        }

        if data.validate(&Default::default()).is_err() {
            return TestResult::discard();
        }

        let mut buffer = vec![0u8; len];

        data.to_buffer(&mut Default::default(), &mut buffer);

        match T::try_from_buffer(&DecoderContext::BASE, &buffer) {
            Ok(d) if d == data => (),
            _ => return TestResult::failed(),
        }

        let mut opt = Some(data.clone());
        let len = <Option<T>>::len(&Default::default());

        let mut buffer = vec![0u8; len];

        opt.to_buffer(&mut Default::default(), &mut buffer);

        match <Option<T>>::try_from_buffer(&DecoderContext::BASE, &buffer) {
            Ok(Some(d)) if d == data => (),
            _ => return TestResult::failed(),
        }

        opt.take();

        opt.to_buffer(&mut Default::default(), &mut buffer);
        match <Option<T>>::try_from_buffer(&DecoderContext::BASE, &buffer) {
            Ok(None) => (),
            _ => return TestResult::failed(),
        }

        TestResult::passed()
    }

    quickcheck(prop as fn(_, ()) -> _);
    quickcheck(prop as fn(_, bool) -> _);
    quickcheck(prop as fn(_, u64) -> _);
    quickcheck(prop as fn(_, usize) -> _);
    quickcheck(prop as fn(_, Config) -> _);
    quickcheck(prop as fn(_, Preamble) -> _);
    quickcheck(prop as fn(_, Scalar) -> _);
    quickcheck(prop as fn(_, Config) -> _);
    quickcheck(prop as fn(_, WiredWitnesses) -> _);
    quickcheck(prop as fn(_, Selectors) -> _);
    quickcheck(prop as fn(_, Polynomial) -> _);
    quickcheck(prop as fn(_, EncodedSource) -> _);
}
