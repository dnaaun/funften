pub mod example;

use yrs::TextPrelim;
use yrs_wrappers::{
    ybox::YBox,
    yrs_basic_types::{YDateTime, YDuration, YUuid},
    yrs_struct::YrsStruct,
    yrs_vec::YrsVecPrelim,
};

#[derive(YrsStruct)]
pub struct PlannedExecutionPrelim {
    pub start: YDateTime,
    pub end: YDateTime,
}

#[derive(YrsStruct)]
pub struct ActualExecutionPrelim {
    pub start: YDateTime,
    pub end: Option<YDateTime>,
}

#[derive(YrsStruct)]
pub struct TodoPrelim {
    pub id: YUuid,
    pub text: TextPrelim<String>,
    pub completed: bool,
    pub created_at: YDateTime,
    pub estimated_duration: YDuration,
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
    use uuid::Uuid;
    use yrs::Doc;
    use yrs::GetString;
    use yrs::Map;
    use yrs::TextPrelim;
    use yrs::Transact;
    use yrs_wrappers::ybox::YBox;

    use super::ActualExecutionPrelim;
    use super::PlannedExecutionPrelim;
    use super::StatePrelim;
    use super::TodoPrelim;

    #[test]
    fn test_new_state() {
        let state_prelim = StatePrelim {
            todos: vec![TodoPrelim {
                id: Uuid::new_v4().into(),
                text: TextPrelim::new("yo".into()),
                completed: false,
                created_at: chrono::Utc::now().naive_utc().into(),
                estimated_duration: chrono::Duration::seconds(60).into(),
                planned_executions: vec![PlannedExecutionPrelim {
                    start: chrono::Utc::now().naive_utc().into(),
                    end: chrono::Utc::now().naive_utc().into(),
                }]
                .into(),
                actual_executions: vec![ActualExecutionPrelim {
                    start: chrono::Utc::now().naive_utc().into(),
                    end: None,
                }]
                .into(),
                child_todos: YBox::new(vec![].into()),
            }]
            .into(),
        };

        let doc = Doc::new();
        let map = doc.get_or_insert_map("state");
        let mut txn = doc.transact_mut();

        let state = map.insert(&mut txn, "state", state_prelim);

        drop(txn);
        let txn = doc.transact();
        let todo_first_text = state
            .todos(&txn)
            .unwrap()
            .get(&txn, 0)
            .unwrap()
            .unwrap()
            .text(&txn)
            .unwrap();
        assert_eq!(todo_first_text.get_string(&txn), "yo");
    }
}
