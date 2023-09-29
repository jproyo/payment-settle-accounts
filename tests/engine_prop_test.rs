use payment_settle_accounts::*;

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    #[test]
    fn test_memory_payment_engine_process(transactions in any::<Vec<Transaction>>()) {
        let mut engine = MemoryPaymentEngine::new();
        for transaction in transactions {
            match engine.process(&transaction) {
                Ok(_) => (),
                Err(e) => {
                    match e {
                        TransactionError::InsufficientFunds => {
                            assert_eq!(transaction.ty(), &TransactionType::Withdrawal);
                        },
                        _ => (),
                    }
                },
            }
        }
        let result = engine.summary();
        for r in result {
            assert!(r.available().as_f64() >= 0.0);
            assert!(r.held().as_f64() >= 0.0);
            assert!(r.total().as_f64() >= 0.0);

            assert_eq!(r.available().as_f64() + r.held().as_f64(), r.total().as_f64());
        }
    }

}
