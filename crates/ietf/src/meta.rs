use crate::DocIdentifier;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait IdContainer {
    type Holder<T>: Serialize + DeserializeOwned + Send + Debug + Clone;

    fn from_inner_text(from: Vec<&str>) -> Vec<Self::Holder<DocIdentifier>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Meta<C>
where
    C: IdContainer,
{
    Updates(Vec<C::Holder<DocIdentifier>>),
    UpdatedBy(Vec<C::Holder<DocIdentifier>>),
    Obsoletes(Vec<C::Holder<DocIdentifier>>),
    ObsoletedBy(Vec<C::Holder<DocIdentifier>>),
    AlsoKnownAs(DocIdentifier),
    Replaces(DocIdentifier),
    Was(DocIdentifier),
}

impl<T> Meta<T>
where
    T: IdContainer,
{
    pub fn from_html(tyype: String, inner_text: Vec<&str>) -> Result<Meta<T>, String> {
        match tyype.as_str() {
            "updated_by" => {
                let updaters = Meta::UpdatedBy(T::from_inner_text(inner_text));
                Ok(updaters)
            }
            "updates" => {
                let updated = Meta::Updates(T::from_inner_text(inner_text));
                Ok(updated)
            }
            "obsoletes" => {
                let obsoleted = Meta::Obsoletes(T::from_inner_text(inner_text));
                Ok(obsoleted)
            }
            "obsoleted_by" => {
                let obsoleters = Meta::ObsoletedBy(T::from_inner_text(inner_text));
                Ok(obsoleters)
            }
            "was" => {
                let was = Meta::Was(inner_text[1].trim().to_string());
                Ok(was)
            }
            "replaces" => {
                let replaced = Meta::Replaces(inner_text[0].trim().to_string());
                Ok(replaced)
            }
            "also_known_as" => {
                let known_as = Meta::AlsoKnownAs(inner_text[1].trim().to_string());
                Ok(known_as)
            }
            _ => Err(format!("Unknown Type {tyype} {{{:#?}}}", inner_text)),
        }
    }
}
