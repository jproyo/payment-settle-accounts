use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::RwLock;

use crate::domain::ClientId;
use crate::domain::Transaction;
use crate::domain::TransactionError;
use crate::domain::TransactionResult;
use crate::domain::TxId;

pub trait PaymentEngine {
    fn process(&mut self, transaction: &Transaction) -> Result<(), TransactionError>;
    fn summary(&self) -> Vec<TransactionResult>;
}

// This storage will contain Deposit or Dispute transaction to keep track of the client's
// disputes, resolves, and chargebacks.
type TxById = HashMap<TxId, RwLock<Vec<Transaction>>>;

/// This storage will contain the current state of the client's account.
type TxByClientId = HashMap<ClientId, RwLock<TransactionResult>>;

pub struct MemoryPaymentEngine {
    tx_state_by_client: Arc<RwLock<TxByClientId>>,
    tx_by_id: Arc<RwLock<TxById>>,
}

impl fmt::Debug for MemoryPaymentEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryPaymentEngine").finish()
    }
}

impl MemoryPaymentEngine {
    pub fn new() -> Self {
        MemoryPaymentEngine {
            tx_state_by_client: Arc::new(RwLock::new(HashMap::new())),
            tx_by_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryPaymentEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PaymentEngine for MemoryPaymentEngine {
    fn process(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        let mut transactions = self.tx_state_by_client.write()?;
        let tx_by_client = transactions
            .entry(transaction.client_id())
            .or_insert_with(|| {
                RwLock::new(
                    TransactionResult::builder()
                        .client_id(transaction.client_id())
                        .available(0)
                        .held(0)
                        .build(),
                )
            });
        let tx_by_client = tx_by_client.get_mut()?;
        // Open a new scope to release the lock on tx_by_client after use it for reading
        {
            let tx_by_id = self.tx_by_id.read()?;
            let tx_by_id_txs = tx_by_id.get(&transaction.transaction_id());
            match tx_by_id_txs {
                Some(txs) => {
                    let txs = txs.read()?;
                    tx_by_client.process(transaction, &txs)?;
                }
                None => tx_by_client.process(transaction, &[])?,
            }
        }
        if transaction.should_be_tracked() {
            let mut tx_by_id = self.tx_by_id.write()?;
            let tx_by_id = tx_by_id
                .entry(transaction.transaction_id())
                .or_insert_with(|| RwLock::new(vec![]));
            tx_by_id.write()?.push(transaction.clone());
        }
        Ok(())
    }

    fn summary(&self) -> Vec<TransactionResult> {
        self.tx_state_by_client
            .read()
            .unwrap()
            .values()
            .map(|tx| tx.read().unwrap().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Transaction;
    use crate::domain::TransactionType;
}
