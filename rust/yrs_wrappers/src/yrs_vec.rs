use crate::yrs_wrapper_error::UnwrapYrsValue;
use crate::yrs_wrapper_error::YrsResult;
use yrs::Array;

use super::try_from_yrs_value::TryFromYrsValue;
use yrs::{block::Prelim, ArrayPrelim, ReadTxn, Transaction, TransactionMut};

pub struct YrsVec<T> {
    inner: yrs::ArrayRef,
    phantom: std::marker::PhantomData<T>,
}

impl<T> TryFromYrsValue for YrsVec<T>
where
    T: TryFromYrsValue,
{
    fn try_from_yrs_value(value: yrs::types::Value, txn: &impl yrs::ReadTxn) -> YrsResult<Self> {
        let array_ref = value.unwrap_yrs_array()?;

        // Verify that the array contains deserializable values.
        array_ref
            .iter(txn)
            .map(|v| T::try_from_yrs_value(v, txn).map(|_| ()))
            .collect::<Result<(), _>>()?;

        Ok(YrsVec {
            inner: array_ref,
            phantom: std::marker::PhantomData,
        })
    }
}

impl<T: TryFromYrsValue> TryFrom<yrs::block::BlockPtr> for YrsVec<T> {
    type Error = <yrs::ArrayRef as TryFrom<yrs::block::BlockPtr>>::Error;

    fn try_from(value: yrs::block::BlockPtr) -> Result<Self, Self::Error> {
        let array_ref: yrs::ArrayRef = value.try_into()?;
        // I'm not sure if it's ok that I don't check that the ArrayRef
        // contains values that are deserializable into T indeed.
        Ok(YrsVec {
            inner: array_ref,
            phantom: std::marker::PhantomData,
        })
    }
}

/// I opt to not parameterize this struct over the type of the inner
/// container (unlike ArrayPrelim), because I don't like the verbosity.
pub struct YrsVecPrelim<P>(ArrayPrelim<Vec<P>, P>);

impl<T, P> yrs::block::Prelim for YrsVecPrelim<P>
where
    P: Prelim<Return = T>,
    T: TryFromYrsValue,
{
    type Return = YrsVec<T>;

    fn into_content(
        self,
        txn: &mut yrs::TransactionMut,
    ) -> (yrs::block::ItemContent, Option<Self>) {
        let (item_content, array_prelim) = self.0.into_content(txn);

        (item_content, array_prelim.map(|a| YrsVecPrelim(a)))
    }

    fn integrate(self, txn: &mut yrs::TransactionMut, inner_ref: yrs::types::BranchPtr) {
        self.0.integrate(txn, inner_ref)
    }
}

impl<V, P> From<V> for YrsVecPrelim<P>
where
    V: IntoIterator<Item = P>,
    ArrayPrelim<Vec<P>, P>: From<V>,
{
    fn from(value: V) -> Self {
        Self(value.into())
    }
}

impl<T> YrsVec<T>
where
    T: TryFromYrsValue,
{
    pub fn iter<'a>(&'a self, txn: &'a impl yrs::ReadTxn) -> impl Iterator<Item = T> + 'a {
        self.inner.iter(txn).map(move |value| {
            T::try_from_yrs_value(value, txn)
                .expect("YrsVec contains values that are not deserializable into T")
        })
    }

    pub fn len(&self, txn: &impl ReadTxn) -> u32 {
        self.inner.len(txn)
    }

    pub fn is_empty(&self, txn: &impl ReadTxn) -> bool {
        self.inner.len(txn) == 0
    }

    pub fn insert<P: Prelim<Return = T>>(
        &self,
        txn: &mut TransactionMut,
        index: u32,
        value: P,
    ) -> P::Return {
        self.inner.insert(txn, index, value)
    }

    pub fn get(&self, txn: &Transaction, index: u32) -> YrsResult<Option<T>> {
        self.inner
            .get(txn, index)
            .map(|value| T::try_from_yrs_value(value, txn))
            .transpose()
    }
}
