use futures::future::BoxFuture;
use parking_lot::MutexGuard;

// pub mod in_mem;

pub type Listener = Box<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync>;

#[async_trait::async_trait]
pub trait Persistence {
    type Error;

    type Subscription;

    async fn get_doc(&self) -> Result<MutexGuard<'_, yrs::Doc>, Self::Error>;

    async fn store_update(&self, update: yrs::Update) -> Result<(), Self::Error>;

    async fn subscribe_to_updates(&self, listener: Listener) -> Self::Subscription;

    async fn unsuscribe_to_updates(
        &self,
        subscription: Self::Subscription,
    ) -> Result<(), Self::Error>;
}
