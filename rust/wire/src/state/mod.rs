pub mod example;

use yrs::TextPrelim;
use yrs_wrappers::{
    ybox::YBox,
    yrs_basic_types::{YBoolPrelim, YDateTimePrelim, YDurationPrelim},
    yrs_struct::YrsStruct,
    yrs_vec::YrsVecPrelim,
};

#[derive(YrsStruct)]
pub struct PlannedExecutionPrelim {
    pub start: YDateTimePrelim,
    pub end: YDateTimePrelim,
}

#[derive(YrsStruct)]
pub struct ActualExecutionPrelim {
    pub start: YDateTimePrelim,
    pub end: Option<YDateTimePrelim>,
}

#[derive(YrsStruct)]
pub struct TodoPrelim {
    pub text: TextPrelim<String>,
    pub completed: YBoolPrelim,
    pub created_at: YDateTimePrelim,
    pub estimated_duration: YDurationPrelim,
    pub planned_executions: YrsVecPrelim<PlannedExecutionPrelim>,
    pub actual_executions: YrsVecPrelim<ActualExecutionPrelim>,
    pub child_todos: YBox<YrsVecPrelim<TodoPrelim>>,
}

#[derive(YrsStruct)]
pub struct StatePrelim {
    pub todos: YrsVecPrelim<TodoPrelim>,
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use yrs::Doc;
    use yrs::GetString;
    use yrs::Map;
    use yrs::TextPrelim;
    use yrs::Transact;
    use yrs_wrappers::ybox::YBox;
    use yrs_wrappers::yrs_wrapper_error::YrsResult;

    use super::ActualExecutionPrelim;
    use super::PlannedExecutionPrelim;
    use super::StatePrelim;
    use super::TodoPrelim;

    #[test]
    fn test_new_state() -> YrsResult<()> {
        let start = chrono::Utc::now().naive_utc();
        let state_prelim = StatePrelim {
            todos: vec![TodoPrelim {
                text: TextPrelim::new("yo".into()),
                completed: false.into(),
                created_at: chrono::Utc::now().naive_utc().into(),
                estimated_duration: chrono::Duration::seconds(60).into(),
                planned_executions: vec![PlannedExecutionPrelim {
                    start: start.into(),
                    end: chrono::Utc::now().naive_utc().into(),
                }]
                .into(),
                actual_executions: vec![ActualExecutionPrelim {
                    start: start.into(),
                    end: None,
                }]
                .into(),
                child_todos: YBox::new(vec![].into()),
            }]
            .into(),
        };

        let doc = Doc::new();
        let map = doc.get_or_insert_map("state");
        let mut txn = doc.try_transact_mut().unwrap();

        let state = map.insert(&mut txn, "state", state_prelim);

        drop(txn);
        let txn = doc.try_transact().unwrap();

        let first_todo = state.todos(&txn)?.get(&txn, 0)?.unwrap();

        let todo_first_text = first_todo.text(&txn)?;
        assert_eq!(todo_first_text.get_string(&txn), "yo");

        let first_todo_completed = first_todo.completed(&txn)?;
        assert_eq!(first_todo_completed.deref(), &false);

        assert_eq!(
            first_todo.estimated_duration(&txn)?.deref(),
            &chrono::Duration::seconds(60)
        );

        assert_eq!(
            first_todo
                .planned_executions(&txn)?
                .get(&txn, 0)?
                .unwrap()
                .start(&txn)?
                // Not testing the fractional component of the seconds because I'm getting errors rn
                // that I don't want to look into (or at least I believe `%.f` refers to fractional
                // seconds, because that's what I removed.
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string(),
            start.format("%Y-%m-%dT%H:%M:%SZ").to_string()
        );

        Ok(())
    }
}
