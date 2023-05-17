use super::try_from_yrs_value::TryFromYrsValue;
use yrs::{block::Prelim, Array, ArrayPrelim, ReadTxn, Transaction, TransactionMut};

pub struct YrsVec<T> {
    inner: yrs::ArrayRef,
    phantom: std::marker::PhantomData<T>,
}

#[derive(Debug, thiserror::Error)]
pub enum YrsVecDeserializeError<E> {
    #[error("Member at index {idx} failed to deserialize with error: {err}")]
    ElementDeserializeError {
        idx: usize,

        #[source]
        err: E,

        #[backtrace]
        backtrace: std::backtrace::Backtrace,
    },

    #[error("expected YArray")]
    ExpectedYArray,
}

impl<T> TryFromYrsValue for YrsVec<T>
where
    T: TryFromYrsValue,
{
    type Error = YrsVecDeserializeError<T::Error>;

    fn try_from_yrs_value(
        value: yrs::types::Value,
        txn: &yrs::Transaction,
    ) -> Result<Self, Self::Error> {
        match value {
            yrs::types::Value::YArray(array_ref) => {
                // Verify that the array contains deserializable values.
                array_ref
                    .iter(txn)
                    .enumerate()
                    .map(|(idx, value)| {
                        T::try_from_yrs_value(value, txn).map_err(|e| (idx, e))?;
                        Ok(())
                    })
                    .collect::<Result<(), (usize, T::Error)>>()
                    .map_err(
                        |(idx, err)| YrsVecDeserializeError::ElementDeserializeError {
                            idx,
                            err,
                            backtrace: std::backtrace::Backtrace::capture(),
                        },
                    )?;

                Ok(YrsVec {
                    inner: array_ref,
                    phantom: std::marker::PhantomData,
                })
            }
            _ => Err(YrsVecDeserializeError::ExpectedYArray),
        }
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
    pub fn iter<'a>(&'a self, txn: &'a yrs::Transaction) -> impl Iterator<Item = T> + 'a {
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

    pub fn get(&self, txn: &Transaction, index: u32) -> Result<Option<T>, T::Error> {
        self.inner
            .get(txn, index)
            .map(|value| T::try_from_yrs_value(value, txn))
            .transpose()
    }
}
