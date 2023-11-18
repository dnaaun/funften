use nutype::nutype;
use std::collections::HashMap;
use std::time::Duration;
use time::OffsetDateTime;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ExternalTimedEntrySync {
    Toggl,
}

pub struct Todo {
    title: String,
    description: String,
}

pub struct ScheduledTimedEntryForRecurring {
    timed_entry: TimedEntry,
    period: Option<PeriodForRecurring>,
    sync_settings: HashMap<ExternalTimedEntrySync, serde_json::Value>,
}

pub struct ScheduledTimedEntryForNonRecurring {
    timed_entry: TimedEntry,
    period: Option<PeriodForNonRecurring>,
}

pub struct TimedEntry {
    duration: Duration,
}

pub struct PeriodForRecurring {
    start: Fraction,

    /// The duration after the start of the period.
    duration: Fraction,
}

pub struct PeriodForNonRecurring {
    start: OffsetDateTime,
    duration: Duration,
}

#[nutype(validate(with = |n| 0.0 <= *n && *n <= 1.0))]
pub struct Fraction(f32);

#[cfg(test)]
mod tests {
    use super::*;
}
