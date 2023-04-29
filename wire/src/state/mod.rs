/// The convention here is `maybe_*` methods do "validations" and return `None` if validation fails.
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};
use std::collections::HashMap;
use yrs::{
    types::Value, Array, ArrayPrelim, GetString, Map, MapPrelim, MapRef, ReadTxn, TextPrelim,
    TransactionMut,
};

fn datetime_from_yrs_value(value: Value) -> Option<DateTime<Utc>> {
    let result = match value {
        Value::Any(lib0::any::Any::BigInt(unix_time)) => {
            Utc.timestamp_millis_opt(unix_time.try_into().ok()?)
        }
        _ => return None,
    };

    match result {
        chrono::LocalResult::Single(datetime) => Some(datetime),
        _ => None,
    }
}

fn duration_from_yrs_value(value: Value) -> Option<Duration> {
    let duration = match value {
        Value::Any(lib0::any::Any::BigInt(duration)) => Duration::milliseconds(duration),
        _ => return None,
    };

    Some(duration)
}

fn bool_from_yrs_value(value: Value) -> Option<bool> {
    let result = match value {
        Value::Any(lib0::any::Any::Bool(b)) => b,
        _ => return None,
    };

    Some(result)
}

/// We use YText, instead of lib0's Any::String, for potenital forward compatiblity with
/// when we want fancy rich text editing.
fn string_from_yrs_value(value: Value, txn: &impl ReadTxn) -> Option<String> {
    let text = match value {
        Value::YText(text) => text,
        _ => return None,
    }
    .get_string(txn);
    Some(text)
}

pub struct PlannedExecutionData {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}
pub struct PlannedExecution(MapRef);

impl PlannedExecution {
    pub fn new(
        map: MapRef,
        txn: &mut TransactionMut,
        data: PlannedExecutionData,
    ) -> PlannedExecution {
        map.insert(
            txn,
            "start",
            lib0::any::Any::BigInt(data.start.timestamp_millis().try_into().unwrap()),
        );
        map.insert(
            txn,
            "end",
            lib0::any::Any::BigInt(data.end.timestamp_millis().try_into().unwrap()),
        );

        PlannedExecution::parse(map, txn).unwrap()
    }

    pub fn parse(map: MapRef, txn: &impl ReadTxn) -> Result<PlannedExecution> {
        let planned_execution = PlannedExecution(map);
        planned_execution.maybe_start(txn).ok_or(anyhow!(
            "`PlannedExecution.start` not found, or of invalid type"
        ))?;
        planned_execution.maybe_end(txn).ok_or(anyhow!(
            "`PlannedExecution.end` not found, or of invalid type"
        ))?;
        Ok(planned_execution)
    }

    fn maybe_start(&self, txn: &impl ReadTxn) -> Option<DateTime<Utc>> {
        datetime_from_yrs_value(self.0.get(txn, "start")?)
    }

    fn maybe_end(&self, txn: &impl ReadTxn) -> Option<DateTime<Utc>> {
        datetime_from_yrs_value(self.0.get(txn, "end")?)
    }

    pub fn start(&self, txn: &impl ReadTxn) -> DateTime<Utc> {
        self.maybe_start(txn).unwrap()
    }

    pub fn end(&self, txn: &impl ReadTxn) -> DateTime<Utc> {
        self.maybe_end(txn).unwrap()
    }
}

pub struct ActualExecutionData {
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

pub struct ActualExecution(MapRef);

impl ActualExecution {
    pub fn new(
        map: MapRef,
        txn: &mut TransactionMut,
        data: ActualExecutionData,
    ) -> ActualExecution {
        map.insert(
            txn,
            "start",
            lib0::any::Any::BigInt(data.start.timestamp_millis().try_into().unwrap()),
        );

        if let Some(end) = data.end {
            map.insert(
                txn,
                "end",
                lib0::any::Any::BigInt(end.timestamp_millis().try_into().unwrap()),
            );
        }

        ActualExecution::parse(map, txn).unwrap()
    }

    pub fn parse(map: MapRef, txn: &impl ReadTxn) -> Result<ActualExecution> {
        let actual_execution = ActualExecution(map);
        actual_execution.maybe_start(txn).ok_or(anyhow!(
            "`ActualExecution.start` not found, or of invalid type"
        ))?;

        // No need to check for `end` because it's optional.

        Ok(actual_execution)
    }

    fn maybe_start(&self, txn: &impl ReadTxn) -> Option<DateTime<Utc>> {
        datetime_from_yrs_value(self.0.get(txn, "start")?)
    }

    fn maybe_end(&self, txn: &impl ReadTxn) -> Option<DateTime<Utc>> {
        datetime_from_yrs_value(self.0.get(txn, "end")?)
    }

    pub fn start(&self, txn: &impl ReadTxn) -> DateTime<Utc> {
        self.maybe_start(txn).unwrap()
    }

    pub fn end(&self, txn: &impl ReadTxn) -> Option<DateTime<Utc>> {
        self.maybe_end(txn)
    }
}

pub struct TodoData {
    text: String,
    completed: bool,
    created_at: DateTime<Utc>,
    estimated_duration: Duration,
    planned_executions: Vec<PlannedExecutionData>,
    actual_executions: Vec<ActualExecutionData>,
}

pub struct Todo(MapRef);

impl Todo {
    pub fn new(map: MapRef, txn: &mut TransactionMut, data: TodoData) -> Todo {
        map.insert(txn, "text", TextPrelim::new(data.text));
        map.insert(txn, "completed", lib0::any::Any::Bool(data.completed));
        map.insert(
            txn,
            "created_at",
            lib0::any::Any::BigInt(data.created_at.timestamp_millis().try_into().unwrap()),
        );
        map.insert(
            txn,
            "estimated_duration",
            lib0::any::Any::BigInt(data.estimated_duration.num_milliseconds()),
        );

        let planned_executions_map = map.insert(
            txn,
            "planned_executions",
            ArrayPrelim::<_, &str>::from([]), // I don't think the // type matters
        );
        data.planned_executions.into_iter().for_each(|p| {
            PlannedExecution::new(
                planned_executions_map.push_back(txn, MapPrelim::<&str>::from(HashMap::new())),
                txn,
                p,
            );
        });

        let actual_executions_map = map.insert(
            txn,
            "actual_executions",
            ArrayPrelim::<_, &str>::from([]), // I don't think the // type matters
        );
        data.actual_executions.into_iter().for_each(|a| {
            ActualExecution::new(
                actual_executions_map.push_back(txn, MapPrelim::<&str>::from(HashMap::new())),
                txn,
                a,
            );
        });

        Todo::parse(map, txn).unwrap()
    }

    /// Verifies that the `map` is the shape that `Todo` expects, and returns a `Todo` if that's
    /// the case.
    pub fn parse(map: MapRef, txn: &impl ReadTxn) -> Result<Todo> {
        let todo = Todo(map);
        todo.maybe_text(txn)
            .ok_or(anyhow!("`Todo.text` not found, or of invalid type"))?;
        todo.maybe_completed(txn)
            .ok_or(anyhow!("`Todo.completed` not found, or of invalid type"))?;
        todo.maybe_created_at(txn)
            .ok_or(anyhow!("`Todo.created_at` not found, or of invalid type"))?;
        todo.maybe_estimated_duration(txn).ok_or(anyhow!(
            "`Todo.estimated_duration` not found, or of invalid type"
        ))?;
        todo.maybe_planned_executions(txn).ok_or(anyhow!(
            "`Todo.planned_executions` not found, or of invalid type"
        ))?;
        todo.maybe_actual_executions(txn).ok_or(anyhow!(
            "`Todo.actual_executions` not found, or of invalid type"
        ))?;
        Ok(todo)
    }

    fn maybe_text(&self, txn: &impl ReadTxn) -> Option<String> {
        let text = self.0.get(txn, "text")?;
        string_from_yrs_value(text, txn)
    }

    fn maybe_completed(&self, txn: &impl ReadTxn) -> Option<bool> {
        let completed = self.0.get(txn, "completed")?;
        bool_from_yrs_value(completed)
    }

    fn maybe_created_at(&self, txn: &impl ReadTxn) -> Option<DateTime<Utc>> {
        datetime_from_yrs_value(self.0.get(txn, "created_at")?)
    }

    fn maybe_estimated_duration(&self, txn: &impl ReadTxn) -> Option<Duration> {
        duration_from_yrs_value(self.0.get(txn, "estimated_duration")?)
    }

    fn maybe_planned_executions(&self, txn: &impl ReadTxn) -> Option<Vec<PlannedExecution>> {
        let planned_executions = self.0.get(txn, "planned_executions")?;
        match planned_executions {
            Value::YArray(planned_executions) => planned_executions
                .iter(txn)
                .map(|p| match p {
                    Value::YMap(p) => PlannedExecution::parse(p, txn).ok(),
                    _ => None,
                })
                .collect(),
            _ => return None,
        }
    }

    fn maybe_actual_executions(&self, txn: &impl ReadTxn) -> Option<Vec<ActualExecution>> {
        let actual_executions = self.0.get(txn, "actual_executions")?;
        match actual_executions {
            Value::YArray(actual_executions) => actual_executions
                .iter(txn)
                .map(|p| match p {
                    Value::YMap(p) => ActualExecution::parse(p, txn).ok(),
                    _ => None,
                })
                .collect(),
            _ => return None,
        }
    }

    pub fn text(&self, txn: &impl ReadTxn) -> String {
        self.maybe_text(txn).unwrap()
    }

    pub fn completed(&self, txn: &impl ReadTxn) -> bool {
        self.maybe_completed(txn).unwrap()
    }

    pub fn created_at(&self, txn: &impl ReadTxn) -> DateTime<Utc> {
        self.maybe_created_at(txn).unwrap()
    }

    pub fn estimated_duration(&self, txn: &impl ReadTxn) -> Duration {
        self.maybe_estimated_duration(txn).unwrap()
    }

    pub fn planned_executions(&self, txn: &impl ReadTxn) -> Vec<PlannedExecution> {
        // We don't follow the usual thing of `maybe_` and `unwrap` because we want to avoid
        // validation.
        if let Value::YArray(value) = self.0.get(txn, "planned_executions").unwrap() {
            value
                .iter(txn)
                .map(|p| match p {
                    Value::YMap(p) => PlannedExecution::parse(p, txn).unwrap(),
                    _ => panic!("`Todo.planned_executions` not found, or of invalid type"),
                })
                .collect()
        } else {
            panic!("`Todo.planned_executions` not found, or of invalid type")
        }
    }

    pub fn actual_executions(&self, txn: &impl ReadTxn) -> Vec<ActualExecution> {
        // We don't follow the usual thing of `maybe_` and `unwrap` because we want to avoid
        // validation.
        if let Value::YArray(value) = self.0.get(txn, "actual_executions").unwrap() {
            value
                .iter(txn)
                .map(|p| match p {
                    Value::YMap(p) => ActualExecution::parse(p, txn).unwrap(),
                    _ => panic!("`Todo.actual_executions` not found, or of invalid type"),
                })
                .collect()
        } else {
            panic!("`Todo.actual_executions` not found, or of invalid type")
        }
    }
}

pub struct State(MapRef);

impl State {
    pub fn new(map: MapRef, txn: &mut TransactionMut, data: Vec<TodoData>) -> State {
        let todos_array = map.insert(txn, "todos", ArrayPrelim::<[&str; 0], &str>::from([]));
        data.into_iter().for_each(|d| {
            let todo_prelim = MapPrelim::<&str>::new();
            let todo_ref = todos_array.push_back(txn, todo_prelim);
            Todo::new(todo_ref, txn, d);
        });

        State::parse(map, txn).unwrap()
    }

    pub fn parse(map: MapRef, txn: &impl ReadTxn) -> Result<State> {
        let state = State(map);
        state
            .maybe_todos(txn)
            .ok_or(anyhow!("`State.todos` not found, or of invalid type"))?;

        Ok(state)
    }

    fn maybe_todos(&self, txn: &impl ReadTxn) -> Option<Vec<Todo>> {
        let todos = self.0.get(txn, "todos")?;
        match todos {
            Value::YArray(todos) => todos
                .iter(txn)
                .map(|p| match p {
                    Value::YMap(p) => Todo::parse(p, txn).ok(),
                    _ => None,
                })
                .collect(),
            _ => return None,
        }
    }

    pub fn todos(&self, txn: &impl ReadTxn) -> Vec<Todo> {
        // We don't follow the usual thing of `maybe_` and `unwrap` because we want to avoid
        // validation.
        if let Value::YArray(value) = self.0.get(txn, "todos").unwrap() {
            value
                .iter(txn)
                .map(|p| match p {
                    Value::YMap(p) => Todo::parse(p, txn).unwrap(),
                    _ => panic!("`State.todos` not found, or of invalid type"),
                })
                .collect()
        } else {
            panic!("`State.todos` not found, or of invalid type")
        }
    }
}

#[cfg(test)]
mod tests {
    use yrs::Doc;
    use yrs::Transact;

    use crate::state::State;

    use super::ActualExecutionData;
    use super::PlannedExecutionData;
    use super::TodoData;
    #[test]
    fn test_new_state() {
        let doc = Doc::new();
        let map = doc.get_or_insert_map("state");
        let mut txn = doc.transact_mut();
        State::new(
            map,
            &mut txn,
            vec![TodoData {
                text: "yo".to_owned(),
                completed: false,
                created_at: chrono::Utc::now(),
                estimated_duration: chrono::Duration::seconds(60),
                planned_executions: vec![PlannedExecutionData {
                    start: chrono::Utc::now(),
                    end: chrono::Utc::now(),
                }],
                actual_executions: vec![ActualExecutionData {
                    start: chrono::Utc::now(),
                    end: None,
                }],
            }],
        );
    }
}
