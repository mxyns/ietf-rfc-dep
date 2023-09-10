use crate::error::{DocError, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceUrl {
    id: String,
    html: Url,
}

impl SourceUrl {
    pub fn html(&self) -> &Url {
        &self.html
    }

    pub fn xml(&self, is_rfc: bool) -> Result<Url> {
        Ok(if is_rfc {
            Url::from_str(format!("https://www.rfc-editor.org/rfc/{}.xml", self.id).as_str())?
        } else {
            Url::from_str(format!("https://www.ietf.org/archive/id/{}.xml", self.id).as_str())?
        })
    }

    pub fn raw(&self, is_rfc: bool) -> Result<Url> {
        Ok(if is_rfc {
            Url::from_str(format!("https://www.rfc-editor.org/rfc/{}.txt", self.id).as_str())?
        } else {
            Url::from_str(format!("https://www.ietf.org/archive/id/{}.txt", self.id).as_str())?
        })
    }

    pub fn new(id: &String) -> Result<Self> {
        Ok(Self {
            id: id.clone(),
            html: Url::from_str(format!("https://datatracker.ietf.org/doc/{}", id).as_str())?,
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
