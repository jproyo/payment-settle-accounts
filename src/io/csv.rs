use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, Stdout};

use serde::Serialize;

use crate::domain::TransactionError;
use crate::{CentDenomination, ClientId, Transaction, TransactionResult};

/// CSVTransactionReader is a wrapper around csv::Reader.
pub struct CSVTransactionReader {
    reader: csv::Reader<BufReader<File>>,
}

/// Implement Debug for CSVTransactionReader hiding details
impl fmt::Debug for CSVTransactionReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CSVTransactionReader")
    }
}

/// CSVReaderIter is a wrapper around csv::DeserializeRecordsIter.
pub struct CSVReaderIter<'a> {
    iter: csv::DeserializeRecordsIter<'a, BufReader<File>, Transaction>,
}

/// Implement Debug for CSVReaderIter hiding details
impl fmt::Debug for CSVReaderIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CSVReaderIter")
    }
}

/// Implement Iterator for CSVReaderIter
impl Iterator for CSVReaderIter<'_> {
    type Item = Result<Transaction, TransactionError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|r| r.map_err(|e| e.into()))
    }
}

impl CSVTransactionReader {
    pub fn iter(&mut self) -> CSVReaderIter<'_> {
        CSVReaderIter {
            iter: self.reader.deserialize(),
        }
    }
}

impl<'a> CSVTransactionReader {
    pub fn new(filename: &'a str) -> Self {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(reader);
        CSVTransactionReader { reader: rdr }
    }
}

#[derive(Debug, Serialize)]
pub struct TransactionResultCSV {
    client: ClientId,
    available: CentDenomination,
    held: CentDenomination,
    total: CentDenomination,
    locked: bool,
}

impl From<TransactionResult> for TransactionResultCSV {
    fn from(result: TransactionResult) -> Self {
        Self {
            client: result.client_id(),
            available: result.available(),
            held: result.held(),
            total: result.total(),
            locked: result.locked(),
        }
    }
}

pub struct CSVTransactionResultStdoutWriter {
    writer: csv::Writer<BufWriter<Stdout>>,
}

impl fmt::Debug for CSVTransactionResultStdoutWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CSVTransactionResultStdoutWriter")
    }
}

impl CSVTransactionResultStdoutWriter {
    pub fn new() -> Self {
        Self {
            writer: csv::Writer::from_writer(BufWriter::new(std::io::stdout())),
        }
    }

    pub fn write<T>(&mut self, result: T) -> Result<(), TransactionError>
    where
        T: Into<TransactionResultCSV>,
    {
        self.writer.serialize(result.into())?;
        Ok(())
    }
}

impl Default for CSVTransactionResultStdoutWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::TransactionType;

    use super::*;

    #[test]
    fn test_csv_reader() {
        let mut csv_reader = CSVTransactionReader::new("tests/data/tx_tests.csv");
        let result = csv_reader.iter().collect::<Result<Vec<Transaction>, _>>();
        assert!(result.is_ok());
        let expected = vec![
            Transaction::builder()
                .ty(TransactionType::Deposit)
                .client_id(1_u16)
                .transaction_id(1_u32)
                .amount(1.0)
                .build(),
            Transaction::builder()
                .ty(TransactionType::Withdrawal)
                .client_id(1_u16)
                .transaction_id(4_u32)
                .amount(1.5)
                .build(),
            Transaction::builder()
                .ty(TransactionType::Dispute)
                .client_id(1_u16)
                .transaction_id(1_u32)
                .build(),
        ];
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_csv_reader_bad_amount() {
        let mut csv_reader = CSVTransactionReader::new("tests/data/tx_tests_bad_amount.csv");
        let result = csv_reader.iter().collect::<Result<Vec<Transaction>, _>>();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid transaction amount Cannot parse \"1.0.0\""));
    }
}
