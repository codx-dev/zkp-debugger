use std::io;

use crate::{CircuitDescription, FixedText, Source};

/// Context without a CDF reader backend
pub type ContextUnit = Context<()>;

/// Encoding and decoding context to be defined by the reader/writer and consumed by the elements
#[derive(Debug, Default)]
pub struct Context<S> {
    source_cache_id: Option<usize>,
    cdf: Option<CircuitDescription<S>>,
}

impl ContextUnit {
    /// Create a new context without CDF backend
    pub const fn unit() -> Self {
        Self {
            source_cache_id: None,
            cdf: None,
        }
    }
}

impl<S> Context<S> {
    /// Set a source cache id in the context. The next witness or constraint encoding should
    /// consume it instead of storing the raw path
    pub fn with_source_cache_id(&mut self, id: usize) -> &mut Self {
        self.source_cache_id.replace(id);
        self
    }

    /// Consume a previously set source cache id.
    pub fn take_source_cache_id(&mut self) -> Option<usize> {
        self.source_cache_id.take()
    }

    /// Set the internal CDF provider
    pub fn with_cdf(cdf: CircuitDescription<S>) -> Self {
        Self {
            source_cache_id: None,
            cdf: Some(cdf),
        }
    }

    /// Fetch the internal source
    pub fn source(&mut self) -> Option<&mut S> {
        self.cdf.as_deref_mut()
    }
}

impl<S> Context<S>
where
    S: io::Read + io::Seek,
{
    /// Attempt to fetch a path from source cache
    pub fn fetch_source_path(&mut self, idx: usize) -> io::Result<FixedText<{ Source::PATH_LEN }>> {
        // The API won't allow the creation of a context with a backend that implements read/seek
        // (hence, different than unit) without a CDF since the only entrypoint is
        // `Context::with_cdf`, and that is never taken.
        self.cdf
            .as_mut()
            .expect("unreachable empty cdf backend")
            .fetch_source(idx)
    }
}
