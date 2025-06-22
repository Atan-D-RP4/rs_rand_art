pub mod bnf_lexer;
pub mod bnf_parser;
pub mod grammar;
pub mod node;

#[cfg(not(target_arch = "wasm32"))]
pub mod simple;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
