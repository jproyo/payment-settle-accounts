#![warn(rust_2018_idioms, missing_debug_implementations)]
mod domain;
mod engine;
mod io;

pub use crate::domain::Transaction;
pub use crate::domain::TransactionType;
pub use crate::io::CSVTransactionReader;
