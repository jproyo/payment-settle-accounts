//! Contains the entities used in the application.

use std::collections::HashMap;

#[cfg(test)]
use fake::Dummy;

use ::serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use typed_builder::TypedBuilder;

use crate::TransactionError;

/// Represents the type of a transaction.
#[derive(Deserialize, PartialEq, Debug, Clone)]
#[cfg_attr(test, derive(Dummy))]
pub enum TransactionType {
    /// Represents a deposit transaction.
    #[serde(rename = "deposit")]
    Deposit,
    /// Represents a withdrawal transaction.
    #[serde(rename = "withdrawal")]
    Withdrawal,
    /// Represents a dispute transaction.
    #[serde(rename = "dispute")]
    Dispute,
    /// Represents a resolve transaction.
    #[serde(rename = "resolve")]
    Resolve,
    /// Represents a chargeback transaction.
    #[serde(rename = "chargeback")]
    Chargeback,
}

/// Represents a client ID.
pub type ClientId = u16;

/// Represents a transaction ID.
pub type TxId = u32;

/// Represents a transaction object.
#[derive(Deserialize, PartialEq, TypedBuilder, Clone, Debug)]
#[cfg_attr(test, derive(Dummy))]
pub struct Transaction {
    #[serde(rename = "type")]
    ty: TransactionType,

    #[serde(rename = "client")]
    client_id: ClientId,

    #[serde(rename = "tx")]
    transaction_id: TxId,

    #[builder(default, setter(strip_option), setter(into))]
    #[serde(rename = "amount")]
    amount: Option<Decimal>,
}

impl Transaction {
    /// Returns the type of the transaction.
    pub fn ty(&self) -> &TransactionType {
        &self.ty
    }

    /// Returns the client ID associated with the transaction.
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Returns the transaction ID.
    pub fn transaction_id(&self) -> u32 {
        self.transaction_id
    }

    /// Returns the amount of the transaction.
    pub fn amount(&self) -> Option<Decimal> {
        self.amount
    }

    /// Returns the amount of the transaction or an error if it is missing.
    pub fn amount_or_err(&self, msg: &str) -> Result<Decimal, TransactionError> {
        let amount = self
            .amount()
            .ok_or_else(|| TransactionError::InvalidTransactionAmount(msg.into()))?;
        if amount < Decimal::ZERO {
            return Err(TransactionError::InvalidTransactionAmount(
                "Transaction amount is negative".into(),
            ));
        }
        Ok(amount)
    }
}

/// Represents the result of a transaction.
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(test, derive(Dummy))]
pub struct Account {
    client_id: ClientId,
    available: Decimal,
    held: Decimal,
    locked: bool,
    being_disputed: bool,
    previous_deposits: HashMap<TxId, Decimal>,
}

impl Account {
    /// Creates a new transaction result with default settle in 0 for `client_id`.
    pub fn new(client_id: ClientId) -> Self {
        Self {
            client_id,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            locked: false,
            being_disputed: false,
            previous_deposits: HashMap::new(),
        }
    }

    pub fn create_with(
        client_id: ClientId,
        available: Decimal,
        held: Decimal,
        locked: bool,
        being_disputed: bool,
    ) -> Self {
        Self {
            client_id,
            available,
            held,
            locked,
            being_disputed,
            previous_deposits: HashMap::new(),
        }
    }

    /// Processes a transaction and updates the transaction result accordingly.
    pub fn process(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        if self.locked {
            return Err(TransactionError::AccountLocked(transaction.clone()));
        }
        match transaction.ty() {
            TransactionType::Deposit => {
                let amount = transaction.amount_or_err("Deposit amount is missing")?;
                if self.exists(transaction) {
                    return Err(TransactionError::DuplicateTransaction(transaction.clone()));
                }
                self.available += amount;
                self.previous_deposits
                    .entry(transaction.transaction_id())
                    .or_insert(amount);
            }
            TransactionType::Withdrawal => {
                let amount = transaction.amount_or_err("Withdrawal amount is missing")?;
                if self.available >= amount {
                    self.available -= amount;
                } else {
                    return Err(TransactionError::InsufficientFunds(transaction.clone()));
                }
            }
            TransactionType::Dispute => {
                if self.being_disputed {
                    return Err(TransactionError::TransactionBeingDisputed(
                        transaction.clone(),
                    ));
                }
                if let Some(&amount) = self.find_previous_deposit(transaction) {
                    if self.available >= amount {
                        self.available -= amount;
                        self.held += amount;
                        self.being_disputed = true;
                    } else {
                        return Err(TransactionError::InconsistenceBalance(
                            "Attempt to dispute more than available".into(),
                            transaction.clone(),
                        ));
                    }
                } else {
                    return Err(TransactionError::CannotDisputeWithoutDeposit(
                        transaction.clone(),
                    ));
                }
            }
            TransactionType::Resolve => {
                if self.being_disputed {
                    if let Some(&amount) = self.find_previous_deposit(transaction) {
                        if self.held >= amount {
                            self.available += amount;
                            self.held -= amount;
                            self.being_disputed = false;
                        } else {
                            return Err(TransactionError::InconsistenceBalance(
                                "Attempt to resolve more than held".into(),
                                transaction.clone(),
                            ));
                        }
                    }
                } else {
                    return Err(TransactionError::CannotResolveWithoutDispute(
                        transaction.clone(),
                    ));
                }
            }
            TransactionType::Chargeback => {
                if self.being_disputed {
                    if let Some(&amount) = self.find_previous_deposit(transaction) {
                        if self.held >= amount {
                            self.held -= amount;
                            self.being_disputed = false;
                            self.locked = true;
                        } else {
                            return Err(TransactionError::InconsistenceBalance(
                                "Attempt to chargeback more than held".into(),
                                transaction.clone(),
                            ));
                        }
                    }
                } else {
                    return Err(TransactionError::CannotChargebackWithoutDispute(
                        transaction.clone(),
                    ));
                }
            }
        }
        Ok(())
    }

    // Verify if the transaction was already processed with same id and type
    fn exists(&self, transaction: &Transaction) -> bool {
        self.previous_deposits
            .get(&transaction.transaction_id())
            .is_some()
    }

    /// Finds the previous deposit transaction for the given transaction.
    fn find_previous_deposit<'a, 'b>(&'a self, transaction: &'b Transaction) -> Option<&Decimal>
    where
        'b: 'a, // 'b lives longer than 'a
    {
        self.previous_deposits.get(&transaction.transaction_id())
    }

    /// Returns the client ID associated with the transaction result.
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Returns the available amount in the transaction result.
    pub fn available(&self) -> Decimal {
        self.available
    }

    /// Returns the held amount in the transaction result.
    pub fn held(&self) -> Decimal {
        self.held
    }

    /// Returns the total amount in the transaction result.
    pub fn total(&self) -> Decimal {
        self.held + self.available
    }

    /// Checks if the transaction result is locked.
    pub fn locked(&self) -> bool {
        self.locked
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(test, derive(Dummy))]
pub struct TransactionResultSummary {
    client: ClientId,
    #[serde(with = "rust_decimal::serde::str")]
    available: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    held: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    total: Decimal,
    locked: bool,
}

impl From<Account> for TransactionResultSummary {
    /// Converts a `TransactionResult` into a `TransactionResultCSV`.
    fn from(result: Account) -> Self {
        Self {
            client: result.client_id(),
            available: result.available().round_dp(4),
            held: result.held().round_dp(4),
            total: result.total().round_dp(4),
            locked: result.locked(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn test_process_deposit() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transaction_result = Account::new(1);

        let result = transaction_result.process(&deposit);

        assert!(result.is_ok());
        let expected = 12.into();
        assert_eq!(transaction_result.available, expected);
    }
    #[test]
    fn test_process_withdrawal_with_sufficient_funds() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transaction_result = Account::new(1);

        let result = transaction_result.process(&deposit);
        assert!(result.is_ok());

        let withdrawal = Transaction::builder()
            .ty(TransactionType::Withdrawal)
            .amount(12)
            .transaction_id(2)
            .client_id(1)
            .build();

        let result = transaction_result.process(&withdrawal);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available, 0.into());
    }

    #[test]
    fn test_process_withdrawal_with_insufficient_funds() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transaction_result = Account::new(1);

        let result = transaction_result.process(&deposit);
        assert!(result.is_ok());

        let withdrawal = Transaction::builder()
            .ty(TransactionType::Withdrawal)
            .amount(dec!(12.1))
            .transaction_id(2)
            .client_id(1)
            .build();

        let result = transaction_result.process(&withdrawal);
        assert!(result.is_err());
        assert_eq!(transaction_result.available, 12.into());
        match result {
            Err(TransactionError::InsufficientFunds(_)) => {}
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn test_process_dispute_with_valid_deposit() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(dec!(12.0))
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transaction_result = Account::new(1);

        let result = transaction_result.process(&deposit);
        assert!(result.is_ok());

        let dispute = Transaction::builder()
            .ty(TransactionType::Dispute)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&dispute);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available(), 0.into());
        assert_eq!(transaction_result.held(), 12.into());
    }

    #[test]
    fn test_process_dispute_with_invalid_deposit() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(dec!(12.0))
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transaction_result = Account::new(1);

        let result = transaction_result.process(&deposit);
        assert!(result.is_ok());

        let dispute = Transaction::builder()
            .ty(TransactionType::Dispute)
            .transaction_id(2)
            .client_id(1)
            .build();

        let result = transaction_result.process(&dispute);
        assert!(result.is_err());
        assert_eq!(transaction_result.available(), 12.into());
        assert_eq!(transaction_result.held(), 0.into());
    }

    #[test]
    fn test_process_resolve_with_valid_dispute() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(dec!(12.0))
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transaction_result = Account::new(1);

        let result = transaction_result.process(&deposit);
        assert!(result.is_ok());

        let dispute = Transaction::builder()
            .ty(TransactionType::Dispute)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&dispute);
        assert!(result.is_ok());

        let resolve = Transaction::builder()
            .ty(TransactionType::Resolve)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&resolve);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available(), 12.into());
        assert_eq!(transaction_result.held(), 0.into());
    }
    #[test]
    fn test_process_dispute_with_not_enough_available() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transaction_result = Account::new(1);

        let result = transaction_result.process(&deposit);
        assert!(result.is_ok());

        let withdrawal = Transaction::builder()
            .ty(TransactionType::Withdrawal)
            .amount(5)
            .transaction_id(2)
            .client_id(1)
            .build();

        let result = transaction_result.process(&withdrawal);
        assert!(result.is_ok());

        let dispute = Transaction::builder()
            .ty(TransactionType::Dispute)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&dispute);
        assert!(result.is_err());
        assert_eq!(transaction_result.available(), 7.into());
        assert_eq!(transaction_result.held(), 0.into());
        match result {
            Err(TransactionError::InconsistenceBalance(..)) => {}
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn test_process_resolve_with_no_dispute() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transaction_result = Account::new(1);

        let result = transaction_result.process(&deposit);
        assert!(result.is_ok());

        let resolve = Transaction::builder()
            .ty(TransactionType::Resolve)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&resolve);
        assert!(result.is_err());
        assert_eq!(transaction_result.available(), 12.into());
        assert_eq!(transaction_result.held(), 0.into());
    }

    #[test]
    fn test_process_resolve_with_no_funds() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transaction_result = Account::new(1);

        let result = transaction_result.process(&deposit);
        assert!(result.is_ok());

        let resolve = Transaction::builder()
            .ty(TransactionType::Resolve)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&resolve);
        assert!(result.is_err());
        assert_eq!(transaction_result.available(), 12.into());
        assert_eq!(transaction_result.held(), 0.into());
    }

    #[test]
    fn test_process_chargeback_with_valid_dispute() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transaction_result = Account::new(1);

        let result = transaction_result.process(&deposit);
        assert!(result.is_ok());

        let dispute = Transaction::builder()
            .ty(TransactionType::Dispute)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&dispute);
        assert!(result.is_ok());

        let chargeback = Transaction::builder()
            .ty(TransactionType::Chargeback)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&chargeback);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available(), 0.into());
        assert_eq!(transaction_result.held(), 0.into());
    }

    #[test]
    fn test_process_chargeback_with_no_dispute() {
        let deposit = Transaction::builder()
            .ty(TransactionType::Deposit)
            .amount(12)
            .transaction_id(1)
            .client_id(1)
            .build();
        let mut transaction_result = Account::new(1);

        let result = transaction_result.process(&deposit);
        assert!(result.is_ok());

        let dispute = Transaction::builder()
            .ty(TransactionType::Dispute)
            .transaction_id(1)
            .client_id(1)
            .build();

        let result = transaction_result.process(&dispute);
        assert!(result.is_ok());

        let chargeback = Transaction::builder()
            .ty(TransactionType::Chargeback)
            .transaction_id(2)
            .client_id(1)
            .build();

        let result = transaction_result.process(&chargeback);
        assert!(result.is_ok());
        assert_eq!(transaction_result.available(), 0.into());
        assert_eq!(transaction_result.held(), 12.into());
    }
}
