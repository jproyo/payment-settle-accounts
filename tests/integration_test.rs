use payment_settle_accounts::*;

#[test]
fn test_process_with_correct_results() {
    let mut csv_reader = CSVTransactionReader::new("tests/data/tx_test_ok.csv");
    let mut engine = MemoryThreadSafePaymentEngine::new();
    for record in csv_reader.iter() {
        engine.process(&record.unwrap()).unwrap();
    }
    let result = engine.summary();
    assert_eq!(result.len(), 2);

    let expected = TransactionResult::builder()
        .client_id(1_u16)
        .available(0.47)
        .held(0.0)
        .locked(false)
        .build();

    let obtained = result.iter().find(|x| x.client_id() == 1_u16).unwrap();

    assert_eq!(obtained, &expected);
}