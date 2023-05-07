use chrono::Duration;

/// A type-safe way to relate phyiscal length with a unit of time.
/// Represents 5 minutes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FiveMins(pub usize);

impl From<Duration> for FiveMins {
    fn from(duration: Duration) -> Self {
        Self(duration.num_minutes() as usize / 5)
    }
}
