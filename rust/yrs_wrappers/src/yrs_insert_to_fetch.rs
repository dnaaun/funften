pub trait YrsInsertToFetch
where
    Self: yrs::block::Prelim,
{
    type Fetch;

    fn insert_to_fetch(
        inserted: Self,
        prelim_return: <Self as yrs::block::Prelim>::Return,
    ) -> Self::Fetch;
}

impl<T> YrsInsertToFetch for T
where
    // This guarantees that Self: yrs::block::Prelim
    T: Into<lib0::any::Any>,
{
    type Fetch = T;

    fn insert_to_fetch(
        inserted: T,
        _prelim_return: <Self as yrs::block::Prelim>::Return,
    ) -> Self::Fetch {
        inserted
    }
}
