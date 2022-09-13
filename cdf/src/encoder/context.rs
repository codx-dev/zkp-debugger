use std::collections::HashMap;
use std::ops::Deref;
use std::{fs, io};

use msgpacker::Message;

use crate::{Config, Preamble};

/// Encoding provider that will convert paths into file contents
pub trait EncoderContextProvider {
    /// Fetch the contents of a given path
    fn contents<P>(&mut self, path: P) -> io::Result<String>
    where
        P: AsRef<str>;
}

/// Default encoding provider with a filesystem backend
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EncoderContextFileProvider;

impl EncoderContextProvider for EncoderContextFileProvider {
    fn contents<P>(&mut self, path: P) -> io::Result<String>
    where
        P: AsRef<str>,
    {
        fs::read_to_string(path.as_ref())
    }
}

impl<V> EncoderContextProvider for HashMap<String, V>
where
    V: AsRef<str>,
{
    fn contents<P>(&mut self, path: P) -> io::Result<String>
    where
        P: AsRef<str>,
    {
        self.get(path.as_ref())
            .map(|p| p.as_ref().to_string())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "the provided path was not found in the disk",
                )
            })
    }
}

/// Context of encoding a CDF file
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EncoderContext {
    preamble: Preamble,
    path_cache: HashMap<String, usize>,
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
        P: Into<String>,
    {
        let path = path.into();
        let len = self.path_cache.len();

        *self.path_cache.entry(path).or_insert(len)
    }
}

impl EncoderContext {
    pub fn write_all<P, W>(&self, mut writer: W, mut provider: P) -> io::Result<usize>
    where
        P: EncoderContextProvider,
        W: io::Write,
    {
        let mut contents = self.path_cache.iter().collect::<Vec<_>>();

        contents.as_mut_slice().sort_by_key(|(_p, i)| *i);

        let paths = contents
            .iter()
            .map(|(p, _i)| p.to_string())
            .collect::<Vec<_>>();

        let contents = paths
            .iter()
            .map(|p| provider.contents(p))
            .map(|p| p.map(Message::String))
            .collect::<io::Result<Vec<_>>>()?;

        let paths = paths.into_iter().map(Message::String).collect();

        let n = Message::Array(paths).pack(&mut writer)?;
        let n = n + Message::Array(contents).pack(&mut writer)?;

        Ok(n)
    }
}

impl Deref for EncoderContext {
    type Target = HashMap<String, usize>;

    fn deref(&self) -> &Self::Target {
        &self.path_cache
    }
}

#[cfg(test)]
use std::path::PathBuf;

#[test]
fn path_cache_is_not_duplicated() {
    let main = PathBuf::from("home")
        .join("zkp-debugger")
        .join("main.rs")
        .display()
        .to_string();

    let lib = PathBuf::from("home")
        .join("zkp-debugger")
        .join("lib.rs")
        .display()
        .to_string();

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
    let main = PathBuf::from("home")
        .join("zkp-debugger")
        .join("main.rs")
        .display()
        .to_string();

    let lib = PathBuf::from("home")
        .join("zkp-debugger")
        .join("lib.rs")
        .display()
        .to_string();

    let mut context = EncoderContext::from_preamble(Default::default());

    let idx_main = context.add_path(main.clone());
    let idx_lib = context.add_path(lib.clone());

    let expected_map: HashMap<String, usize> = vec![(main.clone(), idx_main), (lib, idx_lib)]
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
