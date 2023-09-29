use std::sync::PoisonError;

use thiserror::Error;

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
    #[error("Error parsing CSV file - {0}")]
    CSVError(#[from] csv::Error),
    #[error("Error synchronizing transactions - {0}")]
    SyncError(String),
    #[error("Infusfficient funds for withdrawal")]
    InsufficientFunds,
}

impl<T> From<PoisonError<T>> for TransactionError {
    fn from(value: PoisonError<T>) -> Self {
        TransactionError::SyncError(value.to_string())
    }
}
