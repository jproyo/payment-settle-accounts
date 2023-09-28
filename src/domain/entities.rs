use std::fmt;

use serde::{Deserialize, Serialize, Serializer};
use typed_builder::TypedBuilder;

use super::errors::TransactionError;

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

#[derive(PartialEq, Clone)]
pub struct CentDollars(i64);

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
    fn as_f64(&self) -> f64 {
        self.0 as f64 / 100.0
    }

    fn from_f64(s: f64) -> CentDollars {
        CentDollars((s * 100.0) as i64)
    }
}

impl From<f64> for CentDollars {
    fn from(s: f64) -> CentDollars {
        CentDollars::from_f64(s)
    }
}

#[derive(Deserialize, PartialEq, TypedBuilder, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    ty: TransactionType,

    #[serde(rename = "client")]
    client_id: u16,

    #[serde(rename = "tx")]
    transaction_id: u32,

    #[builder(default, setter(strip_option), setter(into))]
    #[serde(rename = "amount")]
    amount: Option<CentDollars>,
}

impl fmt::Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transaction [type {:?} - client {}  - id {} - amount {:?}]",
            self.ty, self.client_id, self.transaction_id, self.amount
        )
    }
}
