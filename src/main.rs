use env_logger::Env;
use payment_settle_accounts::TransactionPipelineBuilder;
use std::env;

fn main() -> anyhow::Result<()> {
    let filename = match env::args().nth(1) {
        Some(filename) => Ok(filename),
        None => Err(anyhow::format_err!(
            "Usage: {} <csv-complete-filename>",
            env::args().next().unwrap(),
        )),
    }?;

    env_logger::Builder::from_env(Env::default().default_filter_or("error")).init();

    let mut program = TransactionPipelineBuilder::csv_pipeline(filename.as_str());
    program
        .run()
        .map_err(|e| anyhow::anyhow!("Error running transaction pipeline: {}", e))?;
    Ok(())
}
