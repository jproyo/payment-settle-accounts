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

    let expected: TransactionResultSummary =
        Account::create_with(1_u16, dec!(0.4688), dec!(0), false, false).into();
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

    let expected = Account::create_with(1_u16, dec!(0.5), dec!(0), true, false).into();
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
        Account::create_with(1_u16, dec!(80), dec!(0), false, false).into(),
        Account::create_with(2_u16, dec!(80), dec!(0), false, false).into(),
    ];
    result.iter().for_each(|obtained| {
        assert!(expected.contains(obtained));
    });
}
