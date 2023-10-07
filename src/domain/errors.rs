// Error type for the transaction processing
use std::sync::PoisonError;

use thiserror::Error;

use crate::Transaction;

/// Error type for the transaction processing based on thiserror crate
#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Invalid transaction id [{0}]")]
    InvalidTransactionId(String),
    #[error("Invalid client id [{0}]")]
    InvalidClient(String),
    #[error("Invalid transaction type [{0}]")]
    InvalidTransactionType(String),
    #[error("Invalid transaction amount [{0}]")]
    InvalidTransactionAmount(String),
    #[error("Inconsistence Balance amount for transaction [{0} - {1:?}]")]
    InconsistenceBalance(String, Transaction),
    #[error("Error parsing CSV file.\n\n---------------\nOriginal cause:\n---------------\n{0}\n")]
    CSVError(#[from] csv::Error),
    #[error("Error synchronizing transactions\n\n---------------\nOriginal cause:\n---------------\n{0}\n")]
    SyncError(String),
    #[error("Infusfficient funds for withdrawal transaction [{0:?}]")]
    InsufficientFunds(Transaction),
    #[error("Account locked for dispute transaction [{0:?}]")]
    AccountLocked(Transaction),
    #[error("Transaction already processed with same id and type [{0:?}]")]
    DuplicateTransaction(Transaction),
    #[error("Transaction cannot be disputed without a previous deposit [{0:?}]")]
    CannotDisputeWithoutDeposit(Transaction),
    #[error("Transaction cannot be resolved without a dispute [{0:?}]")]
    CannotResolveWithoutDispute(Transaction),
    #[error("Transaction cannot be disputed again because it is under dispute now [{0:?}]")]
    TransactionBeingDisputed(Transaction),
    #[error("Transaction cannot be charged back without a dispute [{0:?}]")]
    CannotChargebackWithoutDispute(Transaction),
}

impl<T> From<PoisonError<T>> for TransactionError {
    fn from(value: PoisonError<T>) -> Self {
        TransactionError::SyncError(value.to_string())
    }
}
