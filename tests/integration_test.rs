use payment_settle_accounts::*;
use rust_decimal_macros::dec;

#[test]
fn test_process_with_correct_results() {
    let mut csv_reader = CSVTransactionReader::new("tests/data/tx_test_ok.csv");
    let mut engine = MemoryThreadSafePaymentEngine::new();
    for record in csv_reader.iter() {
        engine.process(&record.unwrap()).unwrap();
    }
    let result = engine.summary().unwrap().collect::<Vec<_>>();
    assert_eq!(result.len(), 2);

    let expected: TransactionResultSummary = Account::builder()
        .client_id(1_u16)
        .available(dec!(0.4688))
        .held(0)
        .locked(false)
        .build()
        .into();

    assert!(result.contains(&expected));
}

#[test]
fn test_process_with_correct_results_with_chargebacks() {
    let mut csv_reader = CSVTransactionReader::new("tests/data/tx_test_with_charge_back.csv");
    let mut engine = MemoryThreadSafePaymentEngine::new();
    for record in csv_reader.iter() {
        engine.process(&record.unwrap()).unwrap();
    }
    let result = engine.summary().unwrap().collect::<Vec<_>>();
    assert_eq!(result.len(), 2);

    let expected = Account::builder()
        .client_id(1_u16)
        .available(dec!(0.5))
        .held(0)
        .locked(true)
        .build()
        .into();

    assert!(result.contains(&expected));
}

#[test]
fn test_process_with_correct_results_with_chargebacks_and_disputes() {
    let mut csv_reader =
        CSVTransactionReader::new("tests/data/tx_tests_ok_with_dispute_and_chargebacks.csv");
    let mut engine = MemoryThreadSafePaymentEngine::new();
    for record in csv_reader.iter() {
        engine.process(&record.unwrap()).unwrap();
    }
    let result = engine.summary().unwrap().collect::<Vec<_>>();
    assert_eq!(result.len(), 2);

    let expected = vec![
        Account::builder()
            .client_id(1_u16)
            .available(80)
            .held(0)
            .locked(false)
            .build()
            .into(),
        Account::builder()
            .client_id(2_u16)
            .available(80)
            .held(0)
            .locked(false)
            .build()
            .into(),
    ];
    result.iter().for_each(|obtained| {
        assert!(expected.contains(obtained));
    });
}
