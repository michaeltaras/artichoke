#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(missing_docs, intra_doc_link_resolution_failure)]
#![warn(rust_2018_idioms)]

//!  `artichoke-frontend` crate provides binaries for interacting with the
//!  artichoke interpreter in the [`artichoke-backend`](artichoke_backend)
//!  crate.

pub mod parser;
pub mod repl;
pub mod ruby;
