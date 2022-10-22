use crate::Config;

/// Decoding context of a CDF file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecoderContext<'a> {
    config: &'a Config,
    source_names: &'a [String],
    source_contents: &'a [String],
}

impl<'a> DecoderContext<'a> {
    pub(crate) const BASE: Self = Self {
        config: &Config::DEFAULT,
        source_names: &[],
        source_contents: &[],
    };

    pub(crate) const fn new(
        config: &'a Config,
        source_names: &'a [String],
        source_contents: &'a [String],
    ) -> Self {
        Self {
            config,
            source_names,
            source_contents,
        }
    }

    /// Configuration of the decoding.
    pub const fn config(&self) -> &Config {
        self.config
    }

    /// Fetch the name of a file indexed by `id`.
    pub fn fetch_name(&self, id: usize) -> Option<&'a str> {
        self.source_names.get(id).map(|s| s.as_str())
    }

    /// Fetch the contents of a file indexed by `id`.
    pub fn fetch_contents(&self, id: usize) -> Option<&'a str> {
        self.source_contents.get(id).map(|s| s.as_str())
    }
}

#[test]
fn base_is_valid() {
    assert_eq!(
        DecoderContext::new(&Config::default(), &[], &[]),
        DecoderContext::BASE
    );
}
