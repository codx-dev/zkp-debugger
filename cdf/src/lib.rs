#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

//! The binary format for CDF is a dense linear encoding.
//!
//! A circuit is a compositions of items that are either [`Witness`] or [`Constraint`].
//!
//! A [`Witness`] is a constraint system allocated value that is represented by its identifier
//! and [`Scalar`] value.
//!
//! A [`Constraint`] is a [`Polynomial`] expression represented as a gate of the circuit that will
//! allow computation in the constraint system. It will evaluate to a [`bool`] that is the
//! representation of the result of the gate.
//!
//! A circuit description format file will contain a preamble with all its witnesses. Provided
//! this, its witness index will reflect its line on the file, facilitating indexing.

mod config;
mod constraint;
mod decoder;
mod element;
mod encoder;
mod polynomial;
mod preamble;
mod source;
mod witness;

pub use config::{BaseConfig, Config};
pub use constraint::{Constraint, EncodableConstraint};
pub use decoder::{CircuitDescription, DecoderContext};
pub use element::{DecodableElement, Element, EncodableElement, Scalar};
pub use encoder::Encoder;
pub use polynomial::{Polynomial, Selectors, WiredWitnesses};
pub use preamble::Preamble;
pub use source::EncodableSource;
pub use witness::{EncodableWitness, Witness};

pub(crate) mod bytes;
pub(crate) use encoder::EncoderContext;
pub(crate) use source::DecodedSource;
