use std::fmt;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug)]
pub struct CSVTransactionReader<'a> {
    filename: &'a str,
}

pub struct CSVReader {
    reader: csv::Reader<BufReader<File>>,
}

impl fmt::Debug for CSVReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CSVReader")
    }
}

impl Iterator for CSVReader {
    type Item = crate::Transaction;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.reader.deserialize().next();
        match result {
            Some(Ok(record)) => Some(record),
            Some(Err(e)) => {
                println!("Error: {:?}", e);
                None
            }
            None => None,
        }
    }
}

impl<'a> CSVTransactionReader<'a> {
    pub fn new(filename: &'a str) -> Self {
        CSVTransactionReader { filename }
    }

    pub fn read(&self) -> CSVReader {
        let file = File::open(self.filename).unwrap();
        let reader = BufReader::new(file);
        let rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(reader);
        CSVReader { reader: rdr }
    }
}
