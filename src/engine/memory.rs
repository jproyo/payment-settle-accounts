//! Memory implementation of the payment engine.
use log::warn;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::RwLock;

use super::PaymentEngine;
use crate::domain::Account;
use crate::domain::ClientId;
use crate::domain::Transaction;
use crate::domain::TransactionError;
use crate::TransactionResultSummary;

/// This storage will contain the current state of the client's account.
type TxByClientId = HashMap<ClientId, RwLock<Account>>;

/// A thread-safe payment engine that stores transaction information in memory.
/// State is protected by a `RwLock` to allow concurrent reads and exclusive writes in order
/// to speed up the processing of transactions.
#[derive(Clone)]
pub struct MemoryThreadSafePaymentEngine {
    tx_state_by_client: Arc<RwLock<TxByClientId>>,
}

impl fmt::Debug for MemoryThreadSafePaymentEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryPaymentEngine").finish()
    }
}

impl MemoryThreadSafePaymentEngine {
    /// Creates a new `MemoryThreadSafePaymentEngine`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use my_crate::MemoryThreadSafePaymentEngine;
    ///
    /// let engine = MemoryThreadSafePaymentEngine::new();
    /// ```
    pub fn new() -> Self {
        MemoryThreadSafePaymentEngine {
            tx_state_by_client: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryThreadSafePaymentEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PaymentEngine for MemoryThreadSafePaymentEngine {
    /// Processes the given transaction.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to be processed.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the transaction is processed successfully,
    /// otherwise returns a `TransactionError`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use my_crate::{MemoryThreadSafePaymentEngine, Transaction};
    ///
    /// let mut engine = MemoryThreadSafePaymentEngine::new();
    /// let transaction = Transaction::new();
    ///
    /// let result = engine.process(&transaction);
    ///
    /// assert!(result.is_ok());
    /// ```
    fn process(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        let mut transactions = self.tx_state_by_client.write()?;
        let tx_by_client = transactions
            .entry(transaction.client_id())
            .or_insert_with(|| RwLock::new(Account::new(transaction.client_id())));
        let tx_by_client = tx_by_client.get_mut()?;
        match tx_by_client.process(transaction) {
            Ok(_) => {}
            Err(e) => {
                warn!("{}", e);
            }
        }
        Ok(())
    }

    /// Returns a summary of the transaction results.
    ///
    /// # Returns
    ///
    /// Returns a vector of `TransactionResult` representing the summary
    /// of transaction results.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use my_crate::{MemoryThreadSafePaymentEngine, TransactionResult};
    ///
    /// let engine = MemoryThreadSafePaymentEngine::new();
    /// ...
    /// engine.process(tx1);
    /// engine.process(tx2);
    /// engine.process(tx3);
    /// ...
    /// let summary = engine.summary()?;
    /// for result in summary {
    ///    println!("{:?}", result);
    ///    // TransactionResult { client_id: 1, available: 0, held: 0, total: 0, locked: false }
    /// }
    /// ```
    fn summary(
        &self,
    ) -> Result<Box<dyn Iterator<Item = TransactionResultSummary>>, TransactionError> {
        let iter: Vec<TransactionResultSummary> = self
            .tx_state_by_client
            .read()?
            .values()
            .map(|tx| tx.read().unwrap().clone().into())
            .collect();
        Ok(Box::new(iter.into_iter()))
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;
    use crate::*;

    #[test]
    fn test_process() {
        let memory_engine = MemoryThreadSafePaymentEngine::new();

        // Create multiple threads to simultaneously process transactions
        let num_threads = 10;
        let transactions: Vec<Transaction> = vec![
            Transaction::builder()
                .client_id(1)
                .transaction_id(1)
                .amount(1)
                .ty(TransactionType::Deposit)
                .build(),
            Transaction::builder()
                .client_id(1)
                .transaction_id(2)
                .amount(1)
                .ty(TransactionType::Deposit)
                .build(),
            Transaction::builder()
                .client_id(2)
                .transaction_id(1)
                .amount(10)
                .ty(TransactionType::Deposit)
                .build(),
            Transaction::builder()
                .client_id(1)
                .transaction_id(1)
                .ty(TransactionType::Dispute)
                .build(),
            Transaction::builder()
                .client_id(2)
                .transaction_id(4)
                .amount(2)
                .ty(TransactionType::Withdrawal)
                .build(),
            Transaction::builder()
                .client_id(1)
                .transaction_id(1)
                .ty(TransactionType::Chargeback)
                .build(),
        ];

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let mut memory_engine = memory_engine.clone();
                let transactions = transactions.clone();
                thread::spawn(move || {
                    for transaction in transactions {
                        // Call the process method from each thread
                        memory_engine.process(&transaction).unwrap();
                    }
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let memory_engine_summary = memory_engine.clone().summary().unwrap();
        assert_eq!(memory_engine_summary.collect::<Vec<_>>().len(), 2);
    }

    #[test]
    fn test_process_with_existing_tx_by_id() {
        let mut state = MemoryThreadSafePaymentEngine::new();
        let client_id = 1;
        let transaction_id = 1;
        let transaction = Transaction::builder()
            .client_id(client_id)
            .transaction_id(transaction_id)
            .amount(1)
            .ty(TransactionType::Deposit)
            .build();

        let result = state.process(&transaction);

        assert!(result.is_ok());

        let tx_by_client = state.tx_state_by_client.read().unwrap();

        assert_eq!(tx_by_client.len(), 1);
    }

    #[test]
    fn test_process_with_non_existing_tx_by_id() {
        let mut state = MemoryThreadSafePaymentEngine::new();
        let client_id = 1;
        let transaction_id = 1;
        let transaction = Transaction::builder()
            .client_id(client_id)
            .transaction_id(transaction_id)
            .amount(1)
            .ty(TransactionType::Deposit)
            .build();

        let result = state.process(&transaction);

        assert!(result.is_ok());

        let tx_by_client = state.tx_state_by_client.read().unwrap();

        assert_eq!(tx_by_client.len(), 1);
    }

    #[test]
    fn test_process_with_should_be_tracked() {
        let mut state = MemoryThreadSafePaymentEngine::new();
        let client_id = 1;
        let transaction_id = 1;
        let transaction = Transaction::builder()
            .client_id(client_id)
            .transaction_id(transaction_id)
            .amount(1)
            .ty(TransactionType::Deposit)
            .build();

        let result = state.process(&transaction);

        assert!(result.is_ok());
    }

    #[test]
    fn test_process_with_empty_txs() {
        let mut state = MemoryThreadSafePaymentEngine::new();
        let client_id = 1;
        let transaction_id = 1;
        let transaction = Transaction::builder()
            .client_id(client_id)
            .transaction_id(transaction_id)
            .amount(1)
            .ty(TransactionType::Resolve)
            .build();

        let result = state.process(&transaction);

        assert!(result.is_ok());

        let tx_by_client = state.tx_state_by_client.read().unwrap();

        assert_eq!(tx_by_client.len(), 1);
    }
}
