pub trait TryFromYrsValue: Sized {
    type Error: std::error::Error + 'static;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error>;
}

/// TODO: Terrible name
#[derive(Debug, thiserror::Error)]
pub enum NotOriginalTypeError {
    #[error("Did not find a TextRef")]
    TextRef,

    #[error("Did not find a YxmlTextRef")]
    YxmlTextRef,

    #[error("Did not find a YxmlElementRef")]
    YxmlElementRef,

    #[error("Did not find a YxmlFragmentRef")]
    YxmlFragmentRef,

    #[error("Did not find a MapRef")]
    MapRef,

    #[error("Did not find a YDoc")]
    YDoc,

    #[error("Did not find an ArrayRef")]
    ArrayRef,
}

impl TryFromYrsValue for yrs::TextRef {
    type Error = NotOriginalTypeError;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        _txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        match value {
            yrs::types::Value::YText(text_ref) => Ok(text_ref),
            _ => Err(NotOriginalTypeError::TextRef),
        }
    }
}

impl TryFromYrsValue for yrs::XmlTextRef {
    type Error = NotOriginalTypeError;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        _txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        match value {
            yrs::types::Value::YXmlText(yxml_text_ref) => Ok(yxml_text_ref),
            _ => Err(NotOriginalTypeError::YxmlTextRef),
        }
    }
}

impl TryFromYrsValue for yrs::MapRef {
    type Error = NotOriginalTypeError;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        _txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        match value {
            yrs::types::Value::YMap(ymap_ref) => Ok(ymap_ref),
            _ => Err(NotOriginalTypeError::MapRef),
        }
    }
}

impl TryFromYrsValue for yrs::ArrayRef {
    type Error = NotOriginalTypeError;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        _txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        match value {
            yrs::types::Value::YArray(array_ref) => Ok(array_ref),
            _ => Err(NotOriginalTypeError::ArrayRef),
        }
    }
}

impl TryFromYrsValue for yrs::XmlFragmentRef {
    type Error = NotOriginalTypeError;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        _txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        match value {
            yrs::types::Value::YXmlFragment(i) => Ok(i),
            _ => Err(NotOriginalTypeError::YxmlFragmentRef),
        }
    }
}

impl TryFromYrsValue for yrs::XmlElementRef {
    type Error = NotOriginalTypeError;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        _txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        match value {
            yrs::types::Value::YXmlElement(i) => Ok(i),
            _ => Err(NotOriginalTypeError::YxmlElementRef),
        }
    }
}

impl TryFromYrsValue for yrs::Doc {
    type Error = NotOriginalTypeError;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        _txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        match value {
            yrs::types::Value::YDoc(i) => Ok(i),
            _ => Err(NotOriginalTypeError::YDoc),
        }
    }
}

impl TryFromYrsValue for yrs::block::Unused {
    type Error = NotOriginalTypeError;

    fn try_from_yrs_value(
        _value: yrs::types::Value,
        _txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        Ok(yrs::block::Unused)
    }
}

impl<T> TryFromYrsValue for Box<T>
where
    T: TryFromYrsValue,
{
    type Error = T::Error;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        Ok(Box::new(T::try_from_yrs_value(value, txn)?))
    }
}
