#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(intra_doc_link_resolution_failure)]
#![warn(rust_2018_idioms)]

//! `airb` is the Artichoke implementation of `irb` and is an interactive Ruby shell
//! and [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop).
//!
//! `airb` is a readline enabled shell, although it does not persist history.
//!
//! To invoke `airb`, run:
//!
//! ```shell
//! cargo run --bin airb
//! ```

use artichoke_frontend::repl;
use std::io::{self, Write};
use std::process;

fn main() {
    if let Err(err) = repl::run(io::stdout(), io::stderr(), None) {
        let _ = write!(io::stderr(), "{}", err);
        process::exit(1);
    }
}
