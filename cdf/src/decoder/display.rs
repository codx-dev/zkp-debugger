//! Implementations of display for concrete types

use std::fs;

/// Display implementation for the source of a decoder
pub trait DecoderDisplay {
    /// Format the decoder source as string
    fn to_string(&self) -> String;
}

impl DecoderDisplay for fs::File {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
