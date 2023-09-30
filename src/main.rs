use payment_settle_accounts::TransactionPipelineBuilder;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <csv-complete-filename>", args[0]);
        return;
    }

    let filename = &args[1];
    let mut program = TransactionPipelineBuilder::csv_pipeline(filename);
    program
        .run()
        .unwrap_or_else(|e| panic!("Error running transaction pipeline: {}", e));
}
