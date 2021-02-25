//! Implementation of [N-Triples](https://www.w3.org/TR/n-triples/), [N-Quads](https://www.w3.org/TR/n-quads/), [Turtle](https://www.w3.org/TR/turtle/) and [TriG](https://www.w3.org/TR/trig/) parsers.
//!
//! All the provided parsers work in streaming from a `BufRead` implementation.
//! They do not rely on any dependencies outside of Rust standard library.
//! The parsers are not protected against memory overflows.
//! For example if the parsed content contains a literal string of 16 GB, 16 GB of memory will be allocated.
//!
//! How to read a file `foo.ttl` and count the number of `rdf:type` triples:
//! ```no_run
//! use rio_turtle::{TurtleParser, TurtleError};
//! use rio_api::parser::TriplesParser;
//! use rio_api::model::NamedNode;
//! use std::io::BufReader;
//! use std::fs::File;
//! use oxiri::Iri;
//!
//! let rdf_type = NamedNode { iri: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" };
//! let mut count = 0;
//! TurtleParser::new(BufReader::new(File::open("foo.ttl")?), Some(Iri::parse("file:foo.ttl".to_owned()).unwrap())).parse_all(&mut |t| {
//!     if t.predicate == rdf_type {
//!         count += 1;
//!     }
//!     Ok(()) as Result<(), TurtleError>
//! })?;
//! # Result::<_,TurtleError>::Ok(())
//! ```
//!
//! Replace `TurtleParser` by `NTriplesParser`, `NQuadsParser` or `TriGParser` to read an N-Triples, N-Quads or TriG file instead.
//!
//! `NTriplesParser` and `NQuadsParser` do not use the second argument of the `new` function that is the IRI of the file.
//!
//! [Sophia](https://crates.io/crates/sophia_api) adapters for Rio parsers are provided if the `sophia` feature is enabled.
#![deny(
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_qualifications
)]
#![doc(test(attr(deny(warnings))))]

mod error;
mod formatters;
mod ntriples;
mod shared;
mod turtle;
mod utils;

#[cfg(feature = "generalized")]
mod gtrig;

pub use error::TurtleError;
pub use formatters::NQuadsFormatter;
pub use formatters::NTriplesFormatter;
pub use formatters::TriGFormatter;
pub use formatters::TurtleFormatter;
pub use ntriples::NQuadsParser;
pub use ntriples::NTriplesParser;
pub use turtle::TriGParser;
pub use turtle::TurtleParser;

#[cfg(feature = "generalized")]
pub use gtrig::GTriGParser;

#[cfg(feature = "sophia_api")]
mod sophia;
