use payment_settle_accounts::{CSVTransactionReader, MemoryPaymentEngine, PaymentEngine};
use std::env;

fn print_sumary(engine: &MemoryPaymentEngine) {
    let result = engine.summary();
    println!("Summary:");
    for r in result {
        println!("{:?}", r);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <csv-complete-filename>", args[0]);
        return;
    }

    let filename = &args[1];
    let mut csv_reader = CSVTransactionReader::new(filename);
    let mut engine = MemoryPaymentEngine::new();
    for record in csv_reader.iter() {
        println!("---------------------");
        println!("{:?}", record);
        engine.process(&record.unwrap()).unwrap();
    }

    print_sumary(&engine);
}
