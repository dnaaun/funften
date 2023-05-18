use chrono::NaiveDateTime;

use crate::{
    try_from_yrs_value::TryFromYrsValue,
    yrs_wrapper_error::{UnwrapAny, UnwrapYrsValue, YrsResult, YrsWrapperError},
};

#[derive(derive_more::From, Debug)]
pub struct YBoolPrelim(bool);

#[derive(Debug, derive_more::Deref)]
pub struct YBool(bool);

impl yrs::block::Prelim for YBoolPrelim {
    type Return = YBool;

    fn into_content(
        self,
        _txn: &mut yrs::TransactionMut,
    ) -> (yrs::block::ItemContent, Option<Self>) {
        // Copied from Prelim implementation for lib0::any::Any
        let value: lib0::any::Any = self.0.into();
        (yrs::block::ItemContent::Any(vec![value]), None)
    }

    fn integrate(self, _txn: &mut yrs::TransactionMut, _inner_ref: yrs::types::BranchPtr) {}
}

impl TryFromYrsValue for YBool {
    fn try_from_yrs_value(value: yrs::types::Value, _txn: &yrs::Transaction) -> YrsResult<Self> {
        value.unwrap_yrs_any()?.unwrap_any_bool().map(YBool)
    }
}

impl TryFrom<yrs::block::BlockPtr> for YBool {
    type Error = YrsWrapperError;

    fn try_from(value: yrs::block::BlockPtr) -> Result<Self, Self::Error> {
        let any: lib0::any::Any =
            value
                .try_into()
                .map_err(|_| YrsWrapperError::FromBlockPtrError {
                    expected: "lib0::any::Any",
                })?;
        any.unwrap_any_bool().map(YBool)
    }
}

#[derive(derive_more::From, Debug)]
pub struct YDateTimePrelim(NaiveDateTime);

#[derive(Debug, derive_more::Deref)]
pub struct YDateTime(NaiveDateTime);

impl yrs::block::Prelim for YDateTimePrelim {
    type Return = YDateTime;

    fn into_content(
        self,
        _txn: &mut yrs::TransactionMut,
    ) -> (yrs::block::ItemContent, Option<Self>) {
        // Copied from Prelim implementation for lib0::any::Any
        let value: lib0::any::Any = self.0.timestamp_millis().into();
        (yrs::block::ItemContent::Any(vec![value]), None)
    }

    fn integrate(self, _txn: &mut yrs::TransactionMut, _inner_ref: yrs::types::BranchPtr) {}
}

impl TryFromYrsValue for YDateTime {
    fn try_from_yrs_value(value: yrs::types::Value, _txn: &yrs::Transaction) -> YrsResult<Self> {
        let millis = value.unwrap_yrs_any()?.unwrap_any_bigint()?;
        Ok(YDateTime(
            NaiveDateTime::from_timestamp_millis(millis)
                .ok_or_else(|| YrsWrapperError::BigIntOutOfRange)?,
        ))
    }
}

impl TryFrom<yrs::block::BlockPtr> for YDateTime {
    type Error = YrsWrapperError;

    fn try_from(value: yrs::block::BlockPtr) -> Result<Self, Self::Error> {
        let any: lib0::any::Any =
            value
                .try_into()
                .map_err(|_| YrsWrapperError::FromBlockPtrError {
                    expected: "lib0::any::Any",
                })?;

        let millis = any.unwrap_any_bigint()?;
        Ok(YDateTime(
            NaiveDateTime::from_timestamp_millis(millis)
                .ok_or_else(|| YrsWrapperError::BigIntOutOfRange)?,
        ))
    }
}

#[derive(derive_more::From, Debug)]
pub struct YDurationPrelim(chrono::Duration);

#[derive(Debug, derive_more::Deref)]
pub struct YDuration(chrono::Duration);

impl yrs::block::Prelim for YDurationPrelim {
    type Return = YDuration;

    fn into_content(
        self,
        _txn: &mut yrs::TransactionMut,
    ) -> (yrs::block::ItemContent, Option<Self>) {
        // Copied from Prelim implementation for lib0::any::Any
        let value: lib0::any::Any = self.0.num_milliseconds().into();
        (yrs::block::ItemContent::Any(vec![value]), None)
    }

    fn integrate(self, _txn: &mut yrs::TransactionMut, _inner_ref: yrs::types::BranchPtr) {}
}

impl TryFromYrsValue for YDuration {
    fn try_from_yrs_value(value: yrs::types::Value, _txn: &yrs::Transaction) -> YrsResult<Self> {
        let millis = value.unwrap_yrs_any()?.unwrap_any_bigint()?;
        Ok(YDuration(chrono::Duration::milliseconds(millis)))
    }
}

impl TryFrom<yrs::block::BlockPtr> for YDuration {
    type Error = YrsWrapperError;

    fn try_from(value: yrs::block::BlockPtr) -> Result<Self, Self::Error> {
        let any: lib0::any::Any =
            value
                .try_into()
                .map_err(|_| YrsWrapperError::FromBlockPtrError {
                    expected: "lib0::any::Any",
                })?;

        let millis = any.unwrap_any_bigint()?;
        Ok(YDuration(chrono::Duration::milliseconds(millis)))
    }
}
