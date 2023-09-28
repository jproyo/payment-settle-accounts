use std::fmt;
use std::ops::{AddAssign, Deref, Sub, SubAssign};

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

#[derive(PartialEq, Clone, Eq, Hash, PartialOrd, Ord, Copy, Default, Arbitrary)]
pub struct CentDollars(i64);

impl Deref for CentDollars {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AddAssign for CentDollars {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl SubAssign for CentDollars {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl Sub for CentDollars {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        CentDollars(self.0 - other.0)
    }
}

impl fmt::Debug for CentDollars {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.as_f64())
    }
}

impl Serialize for CentDollars {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formatted_value = format!("{:.4}", &self.as_f64());
        serializer.serialize_str(&formatted_value)
    }
}

impl<'de> Deserialize<'de> for CentDollars {
    fn deserialize<D>(deserializer: D) -> Result<CentDollars, D::Error>
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
        Ok(CentDollars::from_f64(s))
    }
}

impl CentDollars {
    pub fn as_f64(&self) -> f64 {
        self.0 as f64 / 100.0
    }

    pub fn from_f64(s: f64) -> CentDollars {
        CentDollars((s * 100.0) as i64)
    }
}

impl From<f64> for CentDollars {
    fn from(s: f64) -> CentDollars {
        CentDollars::from_f64(s)
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

fn prop_valid_amount() -> impl Strategy<Value = Option<CentDollars>> {
    proptest::option::of(0.0..1000.0).prop_map(|x| x.map(CentDollars::from))
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
    amount: Option<CentDollars>,
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

    pub fn amount(&self) -> Option<CentDollars> {
        self.amount
    }

    pub fn amount_or_err(&self, msg: &str) -> Result<CentDollars, TransactionError> {
        self.amount()
            .ok_or_else(|| TransactionError::InvalidTransactionAmount(msg.into()))
    }

    fn is_there_previous_dispute(&self, transaction_result: &[Transaction]) -> bool {
        transaction_result
            .iter()
            .any(|t| t.ty() == &TransactionType::Dispute)
    }

    pub fn should_process(&self, transaction_result: &[Transaction]) -> bool {
        match self.ty() {
            TransactionType::Dispute => !transaction_result
                .iter()
                .any(|t| t.ty() == &TransactionType::Deposit),
            TransactionType::Resolve => self.is_there_previous_dispute(transaction_result),
            TransactionType::Chargeback => self.is_there_previous_dispute(transaction_result),
            _ => true,
        }
    }
}

#[derive(Serialize, PartialEq, TypedBuilder, Clone, Debug)]
pub struct TransactionResult {
    #[serde(rename = "client")]
    client_id: ClientId,
    #[builder(default, setter(into))]
    available: CentDollars,
    #[builder(default, setter(into))]
    held: CentDollars,
    #[builder(default, setter(into))]
    total: CentDollars,
    #[builder(default)]
    locked: bool,
}

impl TransactionResult {
    pub fn process(&mut self, transactions: &[Transaction]) -> Result<(), TransactionError> {
        let mut last_amount_deposit = CentDollars(0);
        for transaction in transactions {
            match transaction.ty() {
                TransactionType::Deposit => {
                    let amount = transaction.amount_or_err("Deposit amount is missing")?;
                    self.available += amount;
                    self.total += amount;
                    last_amount_deposit = amount;
                }
                TransactionType::Withdrawal => {
                    let amount = transaction.amount_or_err("Withdrawal amount is missing")?;
                    if self.available - amount > CentDollars(0) {
                        self.available -= amount;
                        self.total -= amount;
                    }
                }
                TransactionType::Dispute => {
                    self.available -= last_amount_deposit;
                    self.held += last_amount_deposit;
                }
                TransactionType::Resolve => {
                    self.available += last_amount_deposit;
                    self.held -= last_amount_deposit;
                }
                TransactionType::Chargeback => {
                    self.held -= last_amount_deposit;
                    self.total -= last_amount_deposit;
                    self.locked = true;
                }
            }
        }
        Ok(())
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn available(&self) -> CentDollars {
        self.available
    }

    pub fn held(&self) -> CentDollars {
        self.held
    }

    pub fn total(&self) -> CentDollars {
        self.total
    }

    pub fn locked(&self) -> bool {
        self.locked
    }
}
