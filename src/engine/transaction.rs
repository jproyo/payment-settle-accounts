use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use crate::domain::ClientId;
use crate::domain::Transaction;
use crate::domain::TransactionError;
use crate::domain::TransactionResult;
use crate::domain::TxId;

pub trait PaymentEngine {
    fn process(&mut self, transaction: &Transaction) -> Result<(), TransactionError>;
    fn summary(self) -> Vec<TransactionResult>;
}

type TxById = HashMap<TxId, Mutex<Vec<Transaction>>>;
type TxByClientId = HashMap<ClientId, RwLock<TxById>>;

pub struct MemoryPaymentEngine {
    transactions: Arc<RwLock<TxByClientId>>,
}

impl MemoryPaymentEngine {
    pub fn new() -> Self {
        MemoryPaymentEngine {
            transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl PaymentEngine for MemoryPaymentEngine {
    fn process(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        let mut transactions = self.transactions.write().unwrap();
        let tx_by_client = transactions
            .entry(transaction.client_id())
            .or_insert_with(|| RwLock::new(HashMap::new()));
        let tx_by_client = tx_by_client.get_mut().unwrap();
        let transaction_result = tx_by_client
            .entry(transaction.transaction_id())
            .or_insert_with(|| Mutex::new(vec![]));
        let mut transaction_result = transaction_result.lock().unwrap();
        if transaction.should_process(&transaction_result) {
            transaction_result.push(transaction.clone());
        }
        Ok(())
    }

    fn summary(self) -> Vec<TransactionResult> {
        let transactions = self.transactions.read().unwrap();
        transactions.keys().fold(vec![], |mut acc, client_id| {
            let transaction_result = transactions.get(client_id).unwrap().read().unwrap();
            let mut result = TransactionResult::builder()
                .client_id(*client_id)
                .available(0.0)
                .held(0.0)
                .total(0.0)
                .locked(false)
                .build();
            transaction_result.iter().for_each(|(tx_id, txs)| {
                let txs = txs.lock().unwrap();
                result.process(txs.as_slice()).unwrap();
            });
            acc.push(result);
            acc
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Transaction;
    use crate::domain::TransactionType;

    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        #[test]
        fn test_memory_payment_engine_process(transactions in any::<Vec<Transaction>>()) {
            let mut engine = MemoryPaymentEngine::new();
            let len = transactions.len();
            for transaction in transactions {
                let result = engine.process(&transaction);
                assert!(result.is_ok());
            }
            let result = engine.summary();
            println!("{:?}", result);
            assert!(result.len() <= len);
            for r in result {
                assert!(r.available().as_f64() >= 0.0);
                assert!(r.held().as_f64() >= 0.0);
                assert!(r.total().as_f64() >= 0.0);

                assert_eq!(r.available().as_f64() + r.held().as_f64(), r.total().as_f64());
            }
        }

    }
}
