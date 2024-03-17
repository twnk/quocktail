//! Quocktail is a CLI for searching yaml frontmatter in markdown files.
//!
//! Yada yada yada.

mod color;
mod command;
mod output;
mod parse;

// ... other modules

// This is the only export from the crate. It is marked hidden and
// is not part of the public API.
#[doc(hidden)]
pub use command::Quocktail;
