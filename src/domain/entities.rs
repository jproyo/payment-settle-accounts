use std::fmt;
use std::ops::{Add, AddAssign, Deref, Sub, SubAssign};

use serde::{Deserialize, Serialize, Serializer};
use typed_builder::TypedBuilder;

use super::errors::TransactionError;
use proptest::prelude::*;
use proptest_derive::Arbitrary;

#[derive(Deserialize, PartialEq, Debug, Clone, Arbitrary)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    #[proptest(weight = 3)]
    Deposit,
    #[serde(rename = "withdrawal")]
    #[proptest(weight = 1)]
    Withdrawal,
    #[serde(rename = "dispute")]
    #[proptest(weight = 2)]
    Dispute,
    #[serde(rename = "resolve")]
    #[proptest(weight = 2)]
    Resolve,
    #[serde(rename = "chargeback")]
    #[proptest(weight = 1)]
    Chargeback,
}

#[derive(PartialEq, Clone, Eq, Hash, PartialOrd, Ord, Copy, Arbitrary)]
pub struct CentDenomination(i64);

impl Deref for CentDenomination {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AddAssign for CentDenomination {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl SubAssign for CentDenomination {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl Sub for CentDenomination {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        CentDenomination(self.0 - other.0)
    }
}

impl Add for CentDenomination {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        CentDenomination(self.0 + other.0)
    }
}

impl fmt::Debug for CentDenomination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.as_f64())
    }
}

impl Serialize for CentDenomination {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formatted_value = format!("{:.4}", &self.as_f64());
        serializer.serialize_str(&formatted_value)
    }
}

impl<'de> Deserialize<'de> for CentDenomination {
    fn deserialize<D>(deserializer: D) -> Result<CentDenomination, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)
            .map_err(|e| TransactionError::InvalidTransactionAmount(e.to_string()))
            .map_err(serde::de::Error::custom)?;
        let s = s
            .parse::<f64>()
            .map_err(|_| {
                TransactionError::InvalidTransactionAmount(format!("Cannot parse {:?}", s))
            })
            .map_err(serde::de::Error::custom)?;
        Ok(CentDenomination::from_f64(s))
    }
}

impl CentDenomination {
    pub fn as_f64(&self) -> f64 {
        self.0 as f64 / 100.0
    }

    pub fn from_f64(s: f64) -> CentDenomination {
        CentDenomination((s * 100.0) as i64)
    }
}

impl From<f64> for CentDenomination {
    fn from(s: f64) -> CentDenomination {
        CentDenomination::from_f64(s)
    }
}

impl From<i64> for CentDenomination {
    fn from(s: i64) -> CentDenomination {
        CentDenomination(s * 100)
    }
}

pub type ClientId = u16;
pub type TxId = u32;

fn is_valid_tx(tx: &Transaction) -> bool {
    if let TransactionType::Deposit | TransactionType::Withdrawal = tx.ty {
        return tx.amount.is_some();
    }
    true
}

fn prop_valid_amount() -> impl Strategy<Value = Option<CentDenomination>> {
    proptest::option::of(0.0..1000.0).prop_map(|x| x.map(CentDenomination::from))
}

#[derive(Deserialize, PartialEq, TypedBuilder, Clone, Debug, Arbitrary)]
#[proptest(filter = "is_valid_tx")]
pub struct Transaction {
    #[serde(rename = "type")]
    ty: TransactionType,

    #[serde(rename = "client")]
    client_id: ClientId,

    #[serde(rename = "tx")]
    transaction_id: TxId,

    #[builder(default, setter(strip_option), setter(into))]
    #[serde(rename = "amount")]
    #[proptest(strategy = "prop_valid_amount()")]
    amount: Option<CentDenomination>,
}

impl Transaction {
    pub fn ty(&self) -> &TransactionType {
        &self.ty
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn transaction_id(&self) -> u32 {
        self.transaction_id
    }

    pub fn amount(&self) -> Option<CentDenomination> {
        self.amount
    }

    pub fn amount_or_err(&self, msg: &str) -> Result<CentDenomination, TransactionError> {
        self.amount()
            .ok_or_else(|| TransactionError::InvalidTransactionAmount(msg.into()))
    }

    /// We only track deposits and disputes because we don't need to track the rest to check if the
    /// transaction already exists and ther is a dispute or deposit for it.
    pub fn should_be_tracked(&self) -> bool {
        matches!(
            self.ty(),
            TransactionType::Deposit | TransactionType::Dispute
        )
    }

    fn is_there_previous_dispute(&self, transaction_result: &[Transaction]) -> bool {
        transaction_result.iter().any(|t| {
            t.ty() == &TransactionType::Dispute && t.transaction_id() == self.transaction_id()
        })
    }

    fn find_previous_deposit<'a, 'b>(
        &'a self,
        transaction_result: &'b [Transaction],
    ) -> Option<&Transaction>
    where
        'b: 'a,
    {
        transaction_result.iter().find(|t| {
            t.ty() == &TransactionType::Deposit && t.transaction_id() == self.transaction_id()
        })
    }
}

#[derive(Serialize, PartialEq, TypedBuilder, Clone, Debug)]
pub struct TransactionResult {
    #[serde(rename = "client")]
    client_id: ClientId,
    #[builder(setter(into))]
    available: CentDenomination,
    #[builder(setter(into))]
    held: CentDenomination,
    #[builder(default)]
    locked: bool,
}

impl TransactionResult {
    pub fn process(
        &mut self,
        transaction: &Transaction,
        transactions: &[Transaction],
    ) -> Result<(), TransactionError> {
        match transaction.ty() {
            TransactionType::Deposit => {
                let amount = transaction.amount_or_err("Deposit amount is missing")?;
                self.available += amount;
            }
            TransactionType::Withdrawal => {
                let amount = transaction.amount_or_err("Withdrawal amount is missing")?;
                if self.available >= amount {
                    self.available -= amount;
                } else {
                    return Err(TransactionError::InsufficientFunds);
                }
            }
            TransactionType::Dispute => {
                if let Some(deposit) = transaction.find_previous_deposit(transactions) {
                    let amount = deposit.amount_or_err("Deposit amount is missing")?;
                    if self.available >= amount {
                        self.available -= amount;
                        self.held += amount;
                    } else {
                        return Err(TransactionError::InconsistenceBalance(
                            "Attempt to dispute more than available".into(),
                        ));
                    }
                }
            }
            TransactionType::Resolve => {
                if transaction.is_there_previous_dispute(transactions) {
                    if let Some(deposit) = transaction.find_previous_deposit(transactions) {
                        let amount = deposit.amount_or_err("Deposit amount is missing")?;
                        if self.held >= amount {
                            self.available += amount;
                            self.held -= amount;
                        } else {
                            return Err(TransactionError::InconsistenceBalance(
                                "Attempt to resolve more than held".into(),
                            ));
                        }
                    }
                }
            }
            TransactionType::Chargeback => {
                if transaction.is_there_previous_dispute(transactions) {
                    if let Some(deposit) = transaction.find_previous_deposit(transactions) {
                        let amount = deposit.amount_or_err("Deposit amount is missing")?;
                        if self.held >= amount {
                            self.held -= amount;
                            self.locked = true;
                        } else {
                            return Err(TransactionError::InconsistenceBalance(
                                "Attempt to chargeback more than held".into(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn available(&self) -> CentDenomination {
        self.available
    }

    pub fn held(&self) -> CentDenomination {
        self.held
    }

    pub fn total(&self) -> CentDenomination {
        self.held + self.available
    }

    pub fn locked(&self) -> bool {
        self.locked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_cent_denomination() {
        let to_parse = [
            "\"0.5\"",
            "\"0.50\"",
            "\"0.500\"",
            "\"0.5000\"",
            "\"12.2333232\"",
            "\"12.23332\"",
            "\"12.23\"",
            "\"12.2\"",
            "\"0\"",
            "\"0.0\"",
            "\"0.00\"",
            "\"0.000\"",
            "\"0.0000\"",
            "\"2131233212\"",
            "\"2131233\"",
            "\"1233\"",
        ];
        for s in to_parse.iter() {
            let deserializer = &mut serde_json::Deserializer::from_str(s);
            let result = CentDenomination::deserialize(deserializer);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_deserialize_cent_denomination_error() {
        let to_parse = ["\"0.0.\"", "\"0.0.0\"", "\"45.45.45\"", "Not a number"];
        for s in to_parse.iter() {
            let deserializer = &mut serde_json::Deserializer::from_str(s);
            assert!(CentDenomination::deserialize(deserializer).is_err());
        }
    }

    #[test]
    fn test_process_deposit() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12.0)
            .transaction_id(1)
            .client_id(1)
            .build();
        let transactions = vec![];
        let mut transaction_result = TransactionResult::builder()
            .client_id(1)
            .available(0.0)
            .held(0.0)
            .build();

        let result = transaction_result.process(&deposit, &transactions);

        assert!(result.is_ok());
        let expected = 12.into();
        assert_eq!(transaction_result.available, expected);
    }
    #[test]
    fn test_process_withdrawal_with_sufficient_funds() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12.0)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transactions = vec![];
        let mut transaction_result = TransactionResult::builder()
            .client_id(1)
            .available(0.0)
            .held(0.0)
            .build();

        let result = transaction_result.process(&deposit, &transactions);
        assert!(result.is_ok());
        transactions.push(deposit);

        let withdrawal = Transaction::builder()
            .ty(TransactionType::Withdrawal)
            .amount(12)
            .transaction_id(2)
            .client_id(1)
            .build();

        let result = transaction_result.process(&withdrawal, &transactions);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available, 0.into());
    }

    #[test]
    fn test_process_withdrawal_with_insufficient_funds() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12.0)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transactions = vec![];
        let mut transaction_result = TransactionResult::builder()
            .client_id(1)
            .available(0.0)
            .held(0.0)
            .build();

        let result = transaction_result.process(&deposit, &transactions);
        assert!(result.is_ok());
        transactions.push(deposit);

        let withdrawal = Transaction::builder()
            .ty(TransactionType::Withdrawal)
            .amount(12.1)
            .transaction_id(2)
            .client_id(1)
            .build();

        let result = transaction_result.process(&withdrawal, &transactions);
        assert!(result.is_err());
        assert_eq!(transaction_result.available, 12.into());
        match result {
            Err(TransactionError::InsufficientFunds) => {}
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn test_process_dispute_with_valid_deposit() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12.0)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transactions = vec![];
        let mut transaction_result = TransactionResult::builder()
            .client_id(1)
            .available(0.0)
            .held(0.0)
            .build();

        let result = transaction_result.process(&deposit, &transactions);
        assert!(result.is_ok());
        transactions.push(deposit);

        let dispute = Transaction::builder()
            .ty(TransactionType::Dispute)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&dispute, &transactions);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available(), 0.into());
        assert_eq!(transaction_result.held(), 12.into());
    }

    #[test]
    fn test_process_dispute_with_invalid_deposit() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12.0)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transactions = vec![];
        let mut transaction_result = TransactionResult::builder()
            .client_id(1)
            .available(0.0)
            .held(0.0)
            .build();

        let result = transaction_result.process(&deposit, &transactions);
        assert!(result.is_ok());
        transactions.push(deposit);

        let dispute = Transaction::builder()
            .ty(TransactionType::Dispute)
            .transaction_id(2)
            .client_id(1)
            .build();

        let result = transaction_result.process(&dispute, &transactions);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available(), 12.into());
        assert_eq!(transaction_result.held(), 0.into());
    }

    #[test]
    fn test_process_resolve_with_valid_dispute() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12.0)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transactions = vec![];
        let mut transaction_result = TransactionResult::builder()
            .client_id(1)
            .available(0.0)
            .held(0.0)
            .build();

        let result = transaction_result.process(&deposit, &transactions);
        assert!(result.is_ok());
        transactions.push(deposit);

        let dispute = Transaction::builder()
            .ty(TransactionType::Dispute)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&dispute, &transactions);
        assert!(result.is_ok());
        transactions.push(dispute);

        let resolve = Transaction::builder()
            .ty(TransactionType::Resolve)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&resolve, &transactions);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available(), 12.into());
        assert_eq!(transaction_result.held(), 0.into());
    }
    #[test]
    fn test_process_dispute_with_not_enough_available() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12.0)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transactions = vec![];
        let mut transaction_result = TransactionResult::builder()
            .client_id(1)
            .available(0.0)
            .held(0.0)
            .build();

        let result = transaction_result.process(&deposit, &transactions);
        assert!(result.is_ok());
        transactions.push(deposit);

        let withdrawal = Transaction::builder()
            .ty(TransactionType::Withdrawal)
            .amount(5)
            .transaction_id(2)
            .client_id(1)
            .build();

        let result = transaction_result.process(&withdrawal, &transactions);
        assert!(result.is_ok());

        let dispute = Transaction::builder()
            .ty(TransactionType::Dispute)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&dispute, &transactions);
        assert!(result.is_err());
        assert_eq!(transaction_result.available(), 7.into());
        assert_eq!(transaction_result.held(), 0.into());
        match result {
            Err(TransactionError::InconsistenceBalance(_)) => {}
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn test_process_resolve_with_no_dispute() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12.0)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transactions = vec![];
        let mut transaction_result = TransactionResult::builder()
            .client_id(1)
            .available(0.0)
            .held(0.0)
            .build();

        let result = transaction_result.process(&deposit, &transactions);
        assert!(result.is_ok());
        transactions.push(deposit);

        let resolve = Transaction::builder()
            .ty(TransactionType::Resolve)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&resolve, &transactions);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available(), 12.into());
        assert_eq!(transaction_result.held(), 0.into());
    }

    #[test]
    fn test_process_resolve_with_no_funds() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12.0)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transactions = vec![];
        let mut transaction_result = TransactionResult::builder()
            .client_id(1)
            .available(0.0)
            .held(0.0)
            .build();

        let result = transaction_result.process(&deposit, &transactions);
        assert!(result.is_ok());
        transactions.push(deposit);

        let resolve = Transaction::builder()
            .ty(TransactionType::Resolve)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&resolve, &transactions);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available(), 12.into());
        assert_eq!(transaction_result.held(), 0.into());
    }

    #[test]
    fn test_process_chargeback_with_valid_dispute() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12.0)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transactions = vec![];
        let mut transaction_result = TransactionResult::builder()
            .client_id(1)
            .available(0.0)
            .held(0.0)
            .build();

        let result = transaction_result.process(&deposit, &transactions);
        assert!(result.is_ok());
        transactions.push(deposit);

        let dispute = Transaction::builder()
            .ty(TransactionType::Dispute)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&dispute, &transactions);
        assert!(result.is_ok());
        transactions.push(dispute);

        let chargeback = Transaction::builder()
            .ty(TransactionType::Chargeback)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&chargeback, &transactions);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available(), 0.into());
        assert_eq!(transaction_result.held(), 0.into());
    }

    #[test]
    fn test_process_chargeback_with_no_dispute() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12.0)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transactions = vec![];
        let mut transaction_result = TransactionResult::builder()
            .client_id(1)
            .available(0.0)
            .held(0.0)
            .build();

        let result = transaction_result.process(&deposit, &transactions);
        assert!(result.is_ok());
        transactions.push(deposit);

        let dispute = Transaction::builder()
            .ty(TransactionType::Dispute)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&dispute, &transactions);
        assert!(result.is_ok());
        transactions.push(dispute);

        let chargeback = Transaction::builder()
            .ty(TransactionType::Chargeback)
            .transaction_id(2)
            .client_id(1)
            .build();

        let result = transaction_result.process(&chargeback, &transactions);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available(), 0.into());
        assert_eq!(transaction_result.held(), 12.into());
    }
}
