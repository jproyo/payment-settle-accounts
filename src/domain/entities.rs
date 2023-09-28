use std::fmt;

use serde::Deserialize;

mod four_decimal_precision {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formatted_value = format!("{:.4}", value);
        serializer.serialize_str(&formatted_value)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        s.parse::<f64>().map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    ty: String,
    #[serde(rename = "client")]
    client_id: u16,
    #[serde(rename = "tx")]
    transaction_id: u32,
    #[serde(rename = "amount", with = "four_decimal_precision")]
    amount: f64,
}

impl fmt::Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transaction [type {} - client {}  - id {} - amount {:.4}]",
            self.ty, self.client_id, self.transaction_id, self.amount
        )
    }
}
