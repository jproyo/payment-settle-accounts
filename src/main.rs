use payment_settle_accounts::CSVTransactionReader;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <csv-complete-filename>", args[0]);
        return;
    }

    let filename = &args[1];
    let csv_reader = CSVTransactionReader::new(filename);
    for record in csv_reader.read() {
        println!("---------------------");
        println!("{:?}", record);
    }
}
