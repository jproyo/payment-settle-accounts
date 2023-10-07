//! Module that describe domain entities and errors.
mod entities;
mod errors;

pub use entities::Account;
pub use entities::ClientId;
pub use entities::Transaction;
pub use entities::TransactionResultSummary;
pub use entities::TransactionType;
pub use entities::TxId;
pub use errors::*;
