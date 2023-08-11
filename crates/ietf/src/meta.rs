use rfc_dep_cache::CacheReference;
use crate::{doc, DocIdentifier};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Meta {
    Updates(Vec<CacheReference<DocIdentifier>>),
    UpdatedBy(Vec<CacheReference<DocIdentifier>>),
    Obsoletes(Vec<CacheReference<DocIdentifier>>),
    ObsoletedBy(Vec<CacheReference<DocIdentifier>>),
    Was(DocIdentifier),
}

impl Meta {
    pub fn from_html(tyype: String, inner_text: Vec<&str>) -> Result<Meta, String> {
        match tyype.as_str() {
            "updated_by" => {
                let updaters = Meta::UpdatedBy(Self::str_array_to_doc_identifiers(inner_text));
                Ok(updaters)
            }
            "updates" => {
                let updated = Meta::Updates(Self::str_array_to_doc_identifiers(inner_text));
                Ok(updated)
            }
            "obsoletes" => {
                let obsoleted = Meta::Obsoletes(Self::str_array_to_doc_identifiers(inner_text));
                Ok(obsoleted)
            }
            "obsoleted_by" => {
                let obsoleters = Meta::ObsoletedBy(Self::str_array_to_doc_identifiers(inner_text));
                Ok(obsoleters)
            }
            "was" => {
                let was = Meta::Was(inner_text[1].trim().to_string());
                Ok(was)
            }
            _ => {
                Err(format!("Unknown Type {tyype}"))
            }
        }
    }

    fn str_array_to_doc_identifiers(lines: Vec<&str>) -> Vec<CacheReference<DocIdentifier>> {
        lines.into_iter().skip(1).step_by(2).map(|x| {
            CacheReference::Unknown(doc::name_to_id(x))
        }).collect()
    }
}