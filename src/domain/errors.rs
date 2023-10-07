// Error type for the transaction processing
use std::sync::PoisonError;

use thiserror::Error;

use crate::Transaction;

/// Error type for the transaction processing based on thiserror crate
#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Invalid transaction id {0}")]
    InvalidTransactionId(String),
    #[error("Invalid client id {0}")]
    InvalidClient(String),
    #[error("Invalid transaction type {0}")]
    InvalidTransactionType(String),
    #[error("Invalid transaction amount {0}")]
    InvalidTransactionAmount(String),
    #[error("Inconsistence Balance amount for transaction [{0} - {1:?}]")]
    InconsistenceBalance(String, Transaction),
    #[error("Error parsing CSV file.\n\n---------------\nOriginal cause:\n---------------\n{0}\n")]
    CSVError(#[from] csv::Error),
    #[error("Error synchronizing transactions\n\n---------------\nOriginal cause:\n---------------\n{0}\n")]
    SyncError(String),
    #[error("Infusfficient funds for withdrawal transaction {0:?}")]
    InsufficientFunds(Transaction),
}

impl<T> From<PoisonError<T>> for TransactionError {
    fn from(value: PoisonError<T>) -> Self {
        TransactionError::SyncError(value.to_string())
    }
}
