use chrono::Duration;

/// A type-safe way to relate phyiscal length with a unit of time.
/// Represents some length.
#[derive(Clone, Debug, PartialEq, Eq, Hash, derive_more::Deref)]
pub struct TimeLength(usize);

impl From<Duration> for TimeLength {
    fn from(duration: Duration) -> Self {
        Self(duration.num_minutes() as usize / 30)
    }
}
