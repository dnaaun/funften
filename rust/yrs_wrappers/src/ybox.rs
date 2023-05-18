use crate::{try_from_yrs_value::TryFromYrsValue, yrs_wrapper_error::YrsResult};
use yrs::block::{BlockPtr, Prelim};

#[derive(derive_more::From, derive_more::Deref)]
pub struct YBox<T>(Box<T>);

impl<T> TryFromYrsValue for YBox<T>
where
    T: TryFromYrsValue,
{
    fn try_from_yrs_value(value: yrs::types::Value, txn: &impl yrs::ReadTxn) -> YrsResult<Self> {
        Ok(YBox(Box::new(T::try_from_yrs_value(value, txn)?)))
    }
}

impl<T> Prelim for YBox<T>
where
    T: Prelim,
{
    type Return = YBox<T::Return>;

    fn into_content(
        self,
        txn: &mut yrs::TransactionMut,
    ) -> (yrs::block::ItemContent, Option<Self>) {
        let (item_content, thing) = self.0.into_content(txn);
        (item_content, thing.map(|t| YBox(Box::new(t))))
    }

    fn integrate(self, txn: &mut yrs::TransactionMut, inner_ref: yrs::types::BranchPtr) {
        self.0.integrate(txn, inner_ref)
    }
}

impl<T> TryFrom<BlockPtr> for YBox<T>
where
    T: TryFrom<BlockPtr>,
{
    type Error = T::Error;

    fn try_from(value: BlockPtr) -> Result<Self, Self::Error> {
        Ok(YBox(Box::new(T::try_from(value)?)))
    }
}

impl<T> YBox<T> {
    pub fn new(inner: T) -> Self {
        Self(Box::new(inner))
    }
}

