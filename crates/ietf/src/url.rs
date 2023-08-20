use crate::error::{DocError, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceUrl {
    html: Url,
    xml: Url,
}

impl SourceUrl {
    pub fn html(&self) -> &Url {
        &self.html
    }

    pub fn xml(&self) -> &Url {
        &self.xml
    }

    pub fn new(id: &String) -> Result<Self> {
        Ok(Self {
            html: Url::from_str(format!("https://datatracker.ietf.org/doc/{}", id).as_str())?,
            xml: Url::from_str(format!("https://www.ietf.org/archive/id/{}.xml", id).as_str())?,
        })
    }

    pub fn get_id(&self) -> Result<&str> {
        self.html
            .path()
            .rsplit_terminator('/')
            .next()
            .ok_or_else(|| DocError::Url(format!("wrong url {}", self.html)))
    }
}
