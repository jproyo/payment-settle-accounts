#[cfg(test)]
use mockall::{automock, predicate::*};

mod csv;

pub use csv::CSVTransactionReader;
pub use csv::CSVTransactionResultStdoutWriter;

use crate::Transaction;
use crate::TransactionError;
use crate::TransactionResultSummary;

pub trait Source {
    fn read(
        &mut self,
    ) -> Result<
        Box<dyn Iterator<Item = Result<Transaction, TransactionError>> + '_>,
        TransactionError,
    >;
}

impl Source for CSVTransactionReader {
    fn read(
        &mut self,
    ) -> Result<
        Box<dyn Iterator<Item = Result<Transaction, TransactionError>> + '_>,
        TransactionError,
    > {
        Ok(Box::new(self.iter()))
    }
}

#[cfg_attr(test, automock)]
pub trait Sink {
    fn write(&mut self, record: TransactionResultSummary) -> Result<(), TransactionError>;
}

impl Sink for CSVTransactionResultStdoutWriter {
    fn write(&mut self, record: TransactionResultSummary) -> Result<(), TransactionError> {
        self.write(record)
    }
}
