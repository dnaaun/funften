#[derive(Debug, thiserror::Error)]
pub enum YrsWrapperError {
    #[error("Unexpected Yrs value: expected {expected}")]
    UnexpectedYrsValue { expected: &'static str },

    #[error("Attribute {attr} doesn't exist in YMap")]
    YMapMissingAttr { attr: String },

    #[error("Expected variant {expected} in lib0::any::Any")]
    UnexpectedAnyVariant { expected: &'static str },

    #[error("BigInt out of range")]
    BigIntOutOfRange,

    #[error("Failed to convert from BlockPtr to {expected}")]
    FromBlockPtrError { expected: &'static str },
}

pub type YrsResult<T> = Result<T, YrsWrapperError>;

pub trait UnwrapYrsValue {
    fn unwrap_yrs_map(self) -> YrsResult<yrs::MapRef>;
    fn unwrap_yrs_text(self) -> YrsResult<yrs::TextRef>;
    fn unwrap_yrs_array(self) -> YrsResult<yrs::ArrayRef>;
    fn unwrap_yrs_any(self) -> YrsResult<lib0::any::Any>;
}

pub trait UnwrapAny {
    fn unwrap_any_bool(self) -> YrsResult<bool>;
    fn unwrap_any_bigint(self) -> YrsResult<i64>;
    fn unwrap_any_string(self) -> YrsResult<String>;
}

impl UnwrapYrsValue for yrs::types::Value {
    fn unwrap_yrs_map(self) -> YrsResult<yrs::MapRef> {
        match self {
            yrs::types::Value::YMap(map) => Ok(map),
            _ => Err(YrsWrapperError::UnexpectedYrsValue { expected: "MapRef" }),
        }
    }

    fn unwrap_yrs_text(self) -> YrsResult<yrs::TextRef> {
        match self {
            yrs::types::Value::YText(text) => Ok(text),
            _ => Err(YrsWrapperError::UnexpectedYrsValue {
                expected: "TextRef",
            }),
        }
    }

    fn unwrap_yrs_array(self) -> YrsResult<yrs::ArrayRef> {
        match self {
            yrs::types::Value::YArray(array) => Ok(array),
            _ => Err(YrsWrapperError::UnexpectedYrsValue {
                expected: "ArrayRef",
            }),
        }
    }

    fn unwrap_yrs_any(self) -> YrsResult<lib0::any::Any> {
        match self {
            yrs::types::Value::Any(any) => Ok(any),
            _ => Err(YrsWrapperError::UnexpectedYrsValue { expected: "Any" }),
        }
    }
}

impl UnwrapAny for lib0::any::Any {
    fn unwrap_any_bool(self) -> YrsResult<bool> {
        match self {
            lib0::any::Any::Bool(b) => Ok(b),
            _ => Err(YrsWrapperError::UnexpectedAnyVariant { expected: "Bool" }),
        }
    }

    fn unwrap_any_bigint(self) -> YrsResult<i64> {
        match self {
            lib0::any::Any::BigInt(n) => Ok(n),
            _ => Err(YrsWrapperError::UnexpectedAnyVariant { expected: "BigInt" }),
        }
    }

    fn unwrap_any_string(self) -> YrsResult<String> {
        match self {
            lib0::any::Any::String(s) => Ok(s.into()),
            _ => Err(YrsWrapperError::UnexpectedAnyVariant { expected: "String" }),
        }
    }
}
