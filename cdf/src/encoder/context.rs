use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;

use crate::{Config, Preamble};

/// Context of encoding a CDF file
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EncoderContext {
    preamble: Preamble,
    path_cache: HashMap<PathBuf, usize>,
}

impl EncoderContext {
    /// Start a new context
    ///
    /// This function is not intended to be called outside the encoder initialization so we don't
    /// have duplicated contexts
    pub(crate) fn from_preamble(preamble: Preamble) -> Self {
        Self {
            preamble,
            path_cache: HashMap::new(),
        }
    }

    /// Configuration used for the encoding
    pub const fn config(&self) -> &Config {
        &self.preamble.config
    }

    /// Preamble of the context
    pub const fn preamble(&self) -> &Preamble {
        &self.preamble
    }

    /// Append a path to the encoding context, returning its index
    pub fn add_path<P>(&mut self, path: P) -> usize
    where
        P: Into<PathBuf>,
    {
        let path = path.into();
        let len = self.path_cache.len();

        *self.path_cache.entry(path).or_insert(len)
    }
}

impl Deref for EncoderContext {
    type Target = HashMap<PathBuf, usize>;

    fn deref(&self) -> &Self::Target {
        &self.path_cache
    }
}

#[test]
fn path_cache_is_not_duplicated() {
    let main = PathBuf::from("home").join("zkp-debugger").join("main.rs");
    let lib = PathBuf::from("home").join("zkp-debugger").join("lib.rs");

    let mut context = EncoderContext::from_preamble(Default::default());

    let idx_main = context.add_path(main.clone());

    // duplicated path is not added; same index should be returned
    assert_eq!(idx_main, context.add_path(main.clone()));

    let idx_lib = context.add_path(lib.clone());

    // lib is a different path so it should have a different index
    assert_ne!(idx_main, idx_lib);
}

#[test]
fn context_derives_expected_map() {
    let main = PathBuf::from("home").join("zkp-debugger").join("main.rs");
    let lib = PathBuf::from("home").join("zkp-debugger").join("lib.rs");

    let mut context = EncoderContext::from_preamble(Default::default());

    let idx_main = context.add_path(main.clone());
    let idx_lib = context.add_path(lib.clone());

    let expected_map: HashMap<PathBuf, usize> = vec![(main.clone(), idx_main), (lib, idx_lib)]
        .into_iter()
        .collect();

    // deref op should extend map methods to context
    assert_eq!(expected_map[&main], context[&main]);

    // resulting map should equal expect
    assert_eq!(*context, expected_map);
}

#[test]
fn preamble_is_correctly_created() {
    // TODO test all permutations of preamble in integration/fuzz
    let preamble = Preamble::default();
    let context = EncoderContext::from_preamble(preamble);

    // context was created with the right preamble
    assert_eq!(&preamble, context.preamble());
}
