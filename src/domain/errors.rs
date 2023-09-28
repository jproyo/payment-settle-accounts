use thiserror::Error;

use crate::Transaction;

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
    SyncError(#[from] std::sync::PoisonError<std::sync::MutexGuard<'static, Transaction>>),
    #[error("Infusfficient funds for withdrawal")]
    InsufficientFunds,
}
