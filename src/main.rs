use payment_settle_accounts::{
    CSVTransactionReader, CSVTransactionResultStdoutWriter, MemoryPaymentEngine, PaymentEngine,
};
use std::env;

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
        engine.process(&record.unwrap()).unwrap();
    }

    let mut csv_writer = CSVTransactionResultStdoutWriter::new();
    for record in engine.summary() {
        csv_writer.write(record).unwrap();
    }
}
