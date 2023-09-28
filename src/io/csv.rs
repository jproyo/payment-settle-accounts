use std::fmt;
use std::fs::File;
use std::io::BufReader;

use crate::domain::errors::TransactionError;
use crate::Transaction;

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
