use chrono::NaiveDateTime;
use yrs::GetString;

use crate::try_from_yrs_value::TryFromYrsValue;

#[derive(Clone, derive_more::From)]
pub struct YDateTime(chrono::NaiveDateTime);

impl Into<lib0::any::Any> for YDateTime {
    fn into(self) -> lib0::any::Any {
        lib0::any::Any::BigInt(self.0.timestamp_millis())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BasicTypeError {
    #[error("Type can only be deserialized from a {any_variant}")]
    WrongType { any_variant: &'static str },

    #[error("Not of the yrs::types::Value::Any variant")]
    NotAnyVariant,
}

impl TryFromYrsValue for YDateTime {
    type Error = BasicTypeError;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        _txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        match value {
            yrs::types::Value::Any(value) => match value {
                lib0::any::Any::BigInt(millis) => {
                    let native = NaiveDateTime::from_timestamp_millis(millis).unwrap();

                    Ok(YDateTime(native))
                }
                _ => Err(BasicTypeError::WrongType {
                    any_variant: "BigInt",
                }),
            },
            _ => Err(BasicTypeError::NotAnyVariant),
        }
    }
}

#[derive(Clone, derive_more::From)]
pub struct YDuration(chrono::Duration);

impl Into<lib0::any::Any> for YDuration {
    fn into(self) -> lib0::any::Any {
        lib0::any::Any::BigInt(self.0.num_milliseconds())
    }
}

impl TryFromYrsValue for bool {
    type Error = BasicTypeError;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        _txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        match value {
            yrs::types::Value::Any(value) => match value {
                lib0::any::Any::Bool(b) => Ok(b),
                _ => Err(BasicTypeError::WrongType {
                    any_variant: "Bool",
                }),
            },
            _ => Err(BasicTypeError::NotAnyVariant),
        }
    }
}

impl TryFromYrsValue for i64 {
    type Error = BasicTypeError;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        _txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        match value {
            yrs::types::Value::Any(value) => match value {
                lib0::any::Any::BigInt(n) => Ok(n),
                _ => Err(BasicTypeError::WrongType {
                    any_variant: "BigInt",
                }),
            },
            _ => Err(BasicTypeError::NotAnyVariant),
        }
    }
}

#[derive(Clone, derive_more::From)]
pub struct YUuid(uuid::Uuid);

/// Feature gate this
impl TryFromYrsValue for YUuid {
    type Error = BasicTypeError;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        match value {
            yrs::types::Value::YText(text) => {
                let text = text.get_string(txn);
                let uuid = uuid::Uuid::parse_str(&text).unwrap();
                Ok(YUuid(uuid))
            }
            _ => Err(BasicTypeError::NotAnyVariant),
        }
    }
}

impl Into<lib0::any::Any> for YUuid {
    fn into(self) -> lib0::any::Any {
        lib0::any::Any::String(self.0.to_string().into_boxed_str())
    }
}
