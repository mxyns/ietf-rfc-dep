use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use DocError::*;

pub type Result<T> = std::result::Result<T, DocError>;
pub enum DocError {
    Url(String),
    Query(String),
    Lookup(String),
    UnknownMeta(String),
}

impl DocError {
    fn name(&self) -> &'static str {
        match self {
            Url(_) => "UrlError",
            Query(_) => "QueryError",
            Lookup(_) => "LookupError",
            UnknownMeta(_) => "UnknownMetaError",
        }
    }

    fn description(&self) -> &str {
        match self {
            UnknownMeta(s) | Lookup(s) | Query(s) | Url(s) => s.as_str(),
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
        Url(format!("{}", value))
    }
}

impl From<reqwest::Error> for DocError {
    fn from(value: reqwest::Error) -> Self {
        Query(value.to_string())
    }
}
