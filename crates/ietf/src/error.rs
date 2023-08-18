use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use DocError::*;

pub type Result<T> = std::result::Result<T, DocError>;
pub enum DocError {
    UrlError(String),
    QueryError(String),
    LookupError(String),
    UnknownMetaError(String)
}

impl DocError {
    fn name(&self) -> &'static str {
        match self {
            UrlError(_) => { "UrlError" }
            QueryError(_) => { "QueryError" }
            LookupError(_) => { "LookupError" }
            UnknownMetaError(_) => { "UnknownMetaError" }
        }
    }

    fn description(&self) -> &str {
        match self {
            UnknownMetaError(s) | LookupError(s) | QueryError(s) | UrlError(s) => { s.as_str() }
        }
    }
}

impl From<DocError> for String {
    fn from(value: DocError) -> Self {
        value.to_string()
    }
}


impl Debug for DocError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{{{}}}", self.name(), self.description())
    }
}

impl Display for DocError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{{{}}}", self.name(), self.description())
    }
}

impl Error for DocError {}

impl<T> From<DocError> for Result<T> {
    fn from(value: DocError) -> Self {
        Err(value)
    }
}

impl From<url::ParseError> for DocError {
    fn from(value: url::ParseError) -> Self {
        UrlError(format!("{}", value))
    }
}