//! This module contains the definition of the pipeline trait and its implementations for running a
//! program that reads transactions from some `Source`, process them with some `PaymentEngine`, and
//! writes to some `Sink`.
//!
//!
//! Example of composing Pipeline and TransactionPipeline for future implementations like TCPSource and TCPSink.
//!
//! # Example
//!
//! ```no_run
//! use std::net::{TcpStream, TcpListener};
//! use std::io::{BufReader, BufWriter};
//! use std::thread;
//!
//! // Define TCPSource struct implementing Pipeline trait
//! struct TCPSource {
//!     stream: TcpStream,
//! }
//!
//! impl Pipeline for TCPSource {
//!     fn run(&mut self) -> Result<(), TransactionError> {
//!         // Implement TCPSource pipeline logic here
//!         Ok(())
//!     }
//! }
//!
//! // Define TCPSink struct implementing Pipeline trait
//! struct TCPSink {
//!     listener: TcpListener,
//! }
//!
//! impl Pipeline for TCPSink {
//!     fn run(&mut self) -> Result<(), TransactionError> {
//!         // Implement TCPSink pipeline logic here
//!         Ok(())
//!     }
//! }
//!
//! // Compose TransactionPipeline with TCPSource and TCPSink
//! let pipeline: Box<dyn Pipeline> = Box::new(TransactionPipeline {
//!     source: TCPSource { stream: TcpStream::connect("127.0.0.1:8080").unwrap() },
//!     filter: MemoryThreadSafePaymentEngine::new(),
//!     sink: TCPSink { listener: TcpListener::bind("127.0.0.1:8081").unwrap() },
//! });
//! ```
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

#[cfg(test)]
mod tests {

    use fake::{Fake, Faker};
    use mockall::mock;

    use crate::{MockPaymentEngine, MockSink, Transaction, TransactionResultSummary};

    use super::*;

    mock! {
        pub SourceMocked {}
        impl Source for SourceMocked {
            fn read(&mut self) -> Result<Box<dyn Iterator<Item = Result<Transaction, TransactionError>>>, TransactionError>;
        }
    }

    #[test]
    fn test_run_success() {
        let mut source_mock = MockSourceMocked::new();
        let mut filter_mock = MockPaymentEngine::new();
        let mut sink_mock = MockSink::new();

        let returned = fake::vec![Transaction; 3];

        // Set expectations for source mock
        source_mock
            .expect_read()
            .times(1)
            .return_once(|| Ok(Box::new(returned.into_iter().map(Ok))));

        // Set expectations for filter mock
        filter_mock.expect_process().times(3).returning(|_| Ok(()));
        let returned = fake::vec![TransactionResultSummary; 2];
        filter_mock.expect_summary().times(1).return_once(|| {
            Ok(Box::new(returned.into_iter())
                as Box<dyn Iterator<Item = TransactionResultSummary>>)
        });

        // Set expectations for sink mock
        sink_mock.expect_write().times(2).returning(|_| Ok(()));

        let mut transaction_pipeline = Box::new(TransactionPipeline {
            source: source_mock,
            filter: filter_mock,
            sink: sink_mock,
        }) as Box<dyn Pipeline>;

        assert!(transaction_pipeline.run().is_ok());
    }

    #[test]
    fn test_run_source_read_error() {
        let mut source_mock = MockSourceMocked::new();
        let mut filter_mock = MockPaymentEngine::new();
        let mut sink_mock = MockSink::new();

        // Set expectations for source mock
        source_mock.expect_read().times(1).return_once(|| {
            Err(TransactionError::InvalidTransactionAmount(
                "Invliad amount".to_string(),
            ))
        });

        // Set expectations for filter mock
        filter_mock.expect_process().never();
        // Set expectations for sink mock
        sink_mock.expect_write().never();

        let mut transaction_pipeline = Box::new(TransactionPipeline {
            source: source_mock,
            filter: filter_mock,
            sink: sink_mock,
        }) as Box<dyn Pipeline>;

        assert!(transaction_pipeline.run().is_err());
    }
    #[test]
    fn test_run_process_error() {
        let mut source_mock = MockSourceMocked::new();
        let mut filter_mock = MockPaymentEngine::new();
        let mut sink_mock = MockSink::new();

        let returned = fake::vec![Transaction; 3];

        // Set expectations for source mock
        source_mock
            .expect_read()
            .times(1)
            .return_once(|| Ok(Box::new(returned.into_iter().map(Ok))));

        // Set expectations for filter mock
        filter_mock.expect_process().times(1).returning(|_| {
            let tx = Faker.fake();
            Err(TransactionError::InsufficientFunds(tx))
        });
        filter_mock.expect_summary().never();
        // Set expectations for sink mock
        sink_mock.expect_write().never();

        let mut transaction_pipeline = Box::new(TransactionPipeline {
            source: source_mock,
            filter: filter_mock,
            sink: sink_mock,
        }) as Box<dyn Pipeline>;

        assert!(transaction_pipeline.run().is_err());
    }

    #[test]
    fn test_run_summary_error() {
        let mut source_mock = MockSourceMocked::new();
        let mut filter_mock = MockPaymentEngine::new();
        let mut sink_mock = MockSink::new();

        let returned = fake::vec![Transaction; 3];

        // Set expectations for source mock
        source_mock
            .expect_read()
            .times(1)
            .return_once(|| Ok(Box::new(returned.into_iter().map(Ok))));

        // Set expectations for filter mock
        filter_mock.expect_process().times(3).returning(|_| Ok(()));
        filter_mock.expect_summary().times(1).return_once(|| {
            Err(TransactionError::SyncError(
                "Error getting summary".to_string(),
            ))
        });

        // Set expectations for sink mock
        sink_mock.expect_write().never();

        let mut transaction_pipeline = Box::new(TransactionPipeline {
            source: source_mock,
            filter: filter_mock,
            sink: sink_mock,
        }) as Box<dyn Pipeline>;

        assert!(transaction_pipeline.run().is_err());
    }

    #[test]
    fn test_run_sink_error() {
        let mut source_mock = MockSourceMocked::new();
        let mut filter_mock = MockPaymentEngine::new();
        let mut sink_mock = MockSink::new();

        let returned = fake::vec![Transaction; 3];

        // Set expectations for source mock
        source_mock
            .expect_read()
            .times(1)
            .return_once(|| Ok(Box::new(returned.into_iter().map(Ok))));

        // Set expectations for filter mock
        filter_mock.expect_process().times(3).returning(|_| Ok(()));
        let returned = fake::vec![TransactionResultSummary; 2];
        filter_mock.expect_summary().times(1).return_once(|| {
            Ok(Box::new(returned.into_iter())
                as Box<dyn Iterator<Item = TransactionResultSummary>>)
        });

        // Set expectations for sink mock
        sink_mock.expect_write().times(1).return_once(|_| {
            Err(TransactionError::SyncError(
                "Error writing to sink".to_string(),
            ))
        });

        let mut transaction_pipeline = Box::new(TransactionPipeline {
            source: source_mock,
            filter: filter_mock,
            sink: sink_mock,
        }) as Box<dyn Pipeline>;

        assert!(transaction_pipeline.run().is_err());
    }
}
