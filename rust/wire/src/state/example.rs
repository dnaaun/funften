use yrs_wrappers::{yrs_basic_types::YDateTimePrelim, yrs_struct::YrsStruct};


#[derive(YrsStruct)]
pub struct ActualExecutionPrelim {
    pub start: YDateTimePrelim,
    pub end: Option<YDateTimePrelim>,
}
