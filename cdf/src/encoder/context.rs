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
