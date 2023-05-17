/// TODO: I don't retain the error that each attribute has deserializing because
/// That would require that i generate this enum in the derive macro as well.
#[derive(Debug, thiserror::Error)]
pub enum YrsStructDeserializeError {
    #[error("Attribute {attr} failed to deserialize with error: {err}")]
    ElementDeserialize { attr: String, err: Box<dyn std::error::Error> },

    #[error("Missing attribute {attr}")]
    MissingAttribute { attr: String },


    #[error("expected YMap")]
    ExpectedYMap,
}

pub use yrs_struct_derive::YrsStruct;
