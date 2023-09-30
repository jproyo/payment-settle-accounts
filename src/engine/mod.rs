//! Contains the `PaymentEngine` trait definition.
mod memory;

#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::{Transaction, TransactionError, TransactionResult};

/// Trait representing a payment engine. `PaymentEngine` is responsible for processing transactions
/// one by one and keeping track of them in a `TransactionResult` per Client Account.
#[cfg_attr(test, automock)]
pub trait PaymentEngine {
    /// Process a transaction using the payment engine.
    ///
    /// # Arguments
    ///
    /// * `transaction` - A reference to the transaction to be processed.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the transaction was processed successfully, or an `Err` containing
    /// a `TransactionError` if an error occurred during processing.
    fn process(&mut self, transaction: &Transaction) -> Result<(), TransactionError>;

    /// Get a summary of the processed transactions.
    ///
    /// # Returns
    ///
    /// Returns a `Iterator` of `TransactionResult` if there was no error representing the summary of the processed transactions.
    fn summary(&self) -> Result<Box<dyn Iterator<Item = TransactionResult>>, TransactionError>;
}

pub use memory::MemoryThreadSafePaymentEngine;
