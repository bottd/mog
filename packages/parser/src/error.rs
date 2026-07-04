use kdl::KdlError;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
pub enum MogError {
    Simple(char),
    Kdl(String),
    IntegerOverflow(i128),
}

impl From<KdlError> for MogError {
    fn from(e: KdlError) -> Self {
        MogError::Kdl(e.to_string())
    }
}
