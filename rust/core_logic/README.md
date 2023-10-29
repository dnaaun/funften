## Types of constraints we'd like represented for TODO

1. Ideal chunking amount range: Ie, how much of a TODO we SHOULD schedule at one
   point.
2. Total amount of time (in seconds) it should take.
3. Whether it is a time-based, or goal-based activity.
4. Whether it is recurring (weekly, daily, to start with).
5. Deadline: When a TODO item MUST be done by.
    a. For recurring TODOs, deadline is a fractional value indicating the point in time
    between the start and the end of of the recurrence that should be the deadline.
    b. For non recurring TODOs, deadline is a UNIX timestamp.
6. Required time of day: Which times of day we MUST schedule a TODO in. Must be a subset
   of "ideal time", if "ideal time" exists.
    1. For recurring TODOs, it is a set of ranges indicated by fractions.
    2. For non recurring TODOs, it is a set or ranges indicated by time stamps.
7. Ideal time: Which times of day we SHOULD schedule a TODO in.
    1. For recurring TODOs, it is a set of ranges indicated by fractions.
    2. For non recurring TODOs, it is a set or ranges indicated by time stamps.
8. Syncing settings (ie, syncing with Google Tasks, for example.)


## Properties of time entries
1. Which TODO they correspond to.
2. Number of seconds.

### Scheduled Timed Entries
3. Required time of day: Which times of day we MUST schedule a TODO in. Must be a subset
   of "ideal time", if "ideal time" exists. Also must be a subset of the TODO-level
   attribute of the same name.
    1. For recurring TODOs, it is a set of ranges indicated by fractions.
    2. For non recurring TODOs, it is a set or ranges indicated by time stamps.
4. Ideal time: Which times of day we SHOULD schedule a TODO in.
    1. For recurring TODOs, it is a set of ranges indicated by fractions.
    2. For non recurring TODOs, it is a set or ranges indicated by time stamps.


## UX properties
1. An easy way to retroactively say what you did in some past time period.
2. A way to "pause" a certain activity currently going on (and optionally start a new
   one).

## Persistence
Just use [`yrs-persistence`](https://github.com/y-crdt/yrs-persistence/tree/master).
