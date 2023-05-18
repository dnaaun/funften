use yrs_wrappers::yrs_wrapper_error::YrsWrapperError;

pub type GuiResult<T> = Result<T, GuiError>;


#[derive(Debug, thiserror::Error)]
pub enum GuiError {

    #[error("YrsWrapperError: {0}")]
    Yrs(#[from] YrsWrapperError),
}
