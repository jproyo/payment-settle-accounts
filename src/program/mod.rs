use crate::{
    CSVTransactionReader, CSVTransactionResultStdoutWriter, MemoryThreadSafePaymentEngine,
    PaymentEngine, Sink, Source, TransactionError,
};

/// Represents a transaction pipeline, consisting of a source, filter, and sink.
#[derive(Debug)]
pub struct TransactionPipeline<S, F, K> {
    source: S,
    filter: F,
    sink: K,
}

/// Builder for constructing a transaction pipeline.
#[derive(Debug)]
pub struct TransactionPipelineBuilder {}

impl TransactionPipelineBuilder {
    /// Constructs a CSV transaction pipeline.
    ///
    /// # Arguments
    ///
    /// * `filename` - The name of the CSV file to read data from.
    ///
    /// # Returns
    ///
    /// A box containing the constructed pipeline.
    pub fn csv_pipeline(filename: &str) -> Box<dyn Pipeline> {
        Box::new(TransactionPipeline {
            source: CSVTransactionReader::new(filename),
            filter: MemoryThreadSafePaymentEngine::new(),
            sink: CSVTransactionResultStdoutWriter::new(),
        })
    }
}

/// Trait for defining a pipeline.
pub trait Pipeline {
    /// Runs the pipeline.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure of the pipeline.
    fn run(&mut self) -> Result<(), TransactionError>;
}

impl<S, F, K> Pipeline for TransactionPipeline<S, F, K>
where
    S: Source,
    F: PaymentEngine,
    K: Sink,
{
    fn run(&mut self) -> Result<(), TransactionError> {
        let reader = self.source.read()?;
        for record in reader {
            let record = record?;
            self.filter.process(&record)?;
        }
        let results = self.filter.summary()?;
        for record in results {
            self.sink.write(record)?;
        }
        Ok(())
    }
}
