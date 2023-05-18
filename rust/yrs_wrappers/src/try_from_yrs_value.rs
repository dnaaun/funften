use crate::yrs_wrapper_error::{YrsResult, YrsWrapperError};

pub trait TryFromYrsValue: Sized {
    fn try_from_yrs_value(
        value: yrs::types::Value,
        txn: &impl yrs::ReadTxn,
    ) -> Result<Self, YrsWrapperError>;
}

impl TryFromYrsValue for yrs::TextRef {
    fn try_from_yrs_value(value: yrs::types::Value, _txn: &impl yrs::ReadTxn) -> YrsResult<Self> {
        match value {
            yrs::types::Value::YText(text_ref) => Ok(text_ref),
            _ => Err(YrsWrapperError::UnexpectedYrsValue {
                expected: "TextRef",
            }),
        }
    }
}

impl TryFromYrsValue for yrs::XmlTextRef {
    fn try_from_yrs_value(value: yrs::types::Value, _txn: &impl yrs::ReadTxn) -> YrsResult<Self> {
        match value {
            yrs::types::Value::YXmlText(yxml_text_ref) => Ok(yxml_text_ref),
            _ => Err(YrsWrapperError::UnexpectedYrsValue {
                expected: "XmlTextRef",
            }),
        }
    }
}

impl TryFromYrsValue for yrs::MapRef {
    fn try_from_yrs_value(value: yrs::types::Value, _txn: &impl yrs::ReadTxn) -> YrsResult<Self> {
        match value {
            yrs::types::Value::YMap(ymap_ref) => Ok(ymap_ref),
            _ => Err(YrsWrapperError::UnexpectedYrsValue { expected: "MapRef" }),
        }
    }
}

impl TryFromYrsValue for yrs::ArrayRef {
    fn try_from_yrs_value(value: yrs::types::Value, _txn: &impl yrs::ReadTxn) -> YrsResult<Self> {
        match value {
            yrs::types::Value::YArray(array_ref) => Ok(array_ref),
            _ => Err(YrsWrapperError::UnexpectedYrsValue {
                expected: "ArrayRef",
            }),
        }
    }
}

impl TryFromYrsValue for yrs::XmlFragmentRef {
    fn try_from_yrs_value(value: yrs::types::Value, _txn: &impl yrs::ReadTxn) -> YrsResult<Self> {
        match value {
            yrs::types::Value::YXmlFragment(i) => Ok(i),
            _ => Err(YrsWrapperError::UnexpectedYrsValue {
                expected: "XmlFragmentRef",
            }),
        }
    }
}

impl TryFromYrsValue for yrs::XmlElementRef {
    fn try_from_yrs_value(value: yrs::types::Value, _txn: &impl yrs::ReadTxn) -> YrsResult<Self> {
        match value {
            yrs::types::Value::YXmlElement(i) => Ok(i),
            _ => Err(YrsWrapperError::UnexpectedYrsValue {
                expected: "XmlElementRef",
            }),
        }
    }
}

impl TryFromYrsValue for yrs::Doc {
    fn try_from_yrs_value(value: yrs::types::Value, _txn: &impl yrs::ReadTxn) -> YrsResult<Self> {
        match value {
            yrs::types::Value::YDoc(i) => Ok(i),
            _ => Err(YrsWrapperError::UnexpectedYrsValue { expected: "Doc" }),
        }
    }
}

impl<T> TryFromYrsValue for Box<T>
where
    T: TryFromYrsValue,
{
    fn try_from_yrs_value(value: yrs::types::Value, txn: &impl yrs::ReadTxn) -> YrsResult<Self> {
        Ok(Box::new(T::try_from_yrs_value(value, txn)?))
    }
}
