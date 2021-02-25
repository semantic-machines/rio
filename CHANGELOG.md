# Changelog

## [0.5.2] - 2022-02-19

### Added
- `RdfXmlSerializer::with_indentation` to format RDF/XML with indentation

### Changed
- Fixes a bug in RDF/XML formatting where ":" was used in local names
- Upgrades `quick_xml` dependency to 0.21


## [0.5.1] - 2021-01-01

### Changed
- Upgrades `quick_xml` dependency to 0.20
- Improves code style


## [0.5.0] - 2020-08-08

### Added
- Model structures and parsers now implement [Sophia](https://crates.io/crates/sophia_api) traits. This is disabled by default and hidden behind the `sophia` feature.
- `Into<std::io::Error>` is now implemented by `TurtleError` and `RdfXmlError` for easy conversions.

### Removed
- Removes `language_tag` and `iri` from `rio_api`. These modules are now the crates [`oxiri`](https://crates.io/crates/oxiri) and [`oxilangtag`](https://crates.io/crates/oxilangtag).

### Changed
- `base_iri` parameter of parser constructors is now taking values of type `Option<Iri<String>>` instead of `&str`.
- Parser constructors are now returning `Self` and not `Result<Self>`.
- RDF/XML serializers now use `std::io::Error` error type instead of `rio_xml::RdfXmlError` because only I/O related errors might be returned.
- RDF/XML serializer now tries to extract predicate prefixes.
- Use `u64` instead of `usize` to report file positions in order to support parsing big files on 32 bits systems.

## [0.4.2] - 2020-04-04
- Normalizes all language tags to lowercase in the `rio_turtle` and `rio_xml` crates.
- Introduces the `LanguageTag` struct to parse and normalize case of language tags.
- Fixes Turtle parsing when the parser look ahead needs to span multiple lines.
- Makes `Iri` implement `AsRef<str>` and `FromStr`.

## [0.4.1] - 2020-03-19
- Makes `Iri` allow resolving against base IRIs with not hierarchical path (like `file:foo`).
- Upgrades `quick-xml` dependency to 0.18.

## [0.4.0] - 2020-01-07
- Adds "generalized" RDF support and generalized Trig parser behind a "generalized" feature flag.
- Allows to recover NTriples and NQuads parser errors, the parser jumps to the next line if the current line parsing fail.
- Makes `Iri` parser do the full IRI validation.

## [0.3.1] - 2019-09-02

### Added
- `Iri::as_str` and `Display` implementation to `rio_api`.

## [0.3.0] - 2019-08-28

### Added
- `TriplesFormatter` and `QuadsFormatter` with implementations for NTriples, NQuads, Turtle, TriG and RDF XML.
- `Iri` to `rio_api` that allows to do partial IRI validation and resolution.
- `ParseError::textual_position` that allows to get the error position from a `TurtleError` or a `RdfXmlError`.

### Changed
- `TripleParser` have been renamed to `TriplesParser` for consistency.
- `QuadParser` have been renamed to `QuadsParser` for consistency.
- `TriplesParser::parse_step` and `TriplesParser::parse_all` `on_triple` callbacks should now return a `Result`.
  It allows library user to return more easily errors from their callback code.
  The same change have been applied to `QuadsParser`.
- Literals formatting only escape the characters required by the canonical NTriples syntax.

## [0.2.0] - 2019-08-11

### Added
- `Quad` struct and `QuadParser` trait to `rio_api`.
- N-Quads (`NQuadsParser`) and TriG (`TriGParser`) parsers to `rio_turtle`.
- `rdf_xml` crate with an RDF XML parser.

### Changed
- `\r` characters could also end comments in Turtle/TriG.
- Fixes IRI parsing when the IRI has an authority and a query and/or fragment but no path.
- Do not allow "[] ." lines in Turtle.
- Minor optimisations to the Turtle parser.

## [0.1.0] - 2019-07-28

### Added
- `rio_api` crate with `Triple` struct and `TripleParser` trait.
- `rio_turtle` crate with N-Triples (`TurtleParser`) and Turtle (`TurtleParser`).
