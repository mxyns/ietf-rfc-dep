use crate::error::DocError::UnknownMeta;
use crate::error::Result;
use crate::{name_to_id, DocIdentifier};
use fast_xml::events::attributes::Attribute;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait IdContainer {
    type Holder<T>: Serialize + DeserializeOwned + Send + Debug + Clone + From<DocIdentifier>;
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
    Replaces(C::Holder<DocIdentifier>),
    ReplacedBy(C::Holder<DocIdentifier>),
    Was(DocIdentifier),
}

impl<C> Meta<C>
where
    C: IdContainer,
{
    fn from_inner_text(lines: Vec<&str>) -> Vec<C::Holder<DocIdentifier>> {
        lines
            .into_iter()
            .skip(1)
            .step_by(2)
            .map(|x| C::Holder::from(name_to_id(x)))
            .collect()
    }
    pub fn from_html(tyype: String, inner_text: Vec<&str>) -> Result<Meta<C>> {
        match tyype.as_str() {
            "updated_by" => {
                let updaters = Meta::UpdatedBy(Self::from_inner_text(inner_text));
                Ok(updaters)
            }
            "updates" => {
                let updated = Meta::Updates(Self::from_inner_text(inner_text));
                Ok(updated)
            }
            "obsoletes" => {
                let obsoleted = Meta::Obsoletes(Self::from_inner_text(inner_text));
                Ok(obsoleted)
            }
            "obsoleted_by" => {
                let obsoleters = Meta::ObsoletedBy(Self::from_inner_text(inner_text));
                Ok(obsoleters)
            }
            "was" => {
                let was = Meta::Was(name_to_id(inner_text[1].trim().to_string()));
                Ok(was)
            }
            "replaces" => {
                let replaced = Meta::Replaces(C::Holder::from(name_to_id(
                    inner_text[0].trim().to_string(),
                )));
                Ok(replaced)
            }
            "replaced_by" => {
                let replacer = Meta::ReplacedBy(C::Holder::from(name_to_id(
                    inner_text[0].trim().to_string(),
                )));
                Ok(replacer)
            }
            "also_known_as" => {
                let known_as = Meta::AlsoKnownAs(name_to_id(inner_text[1].trim().to_string()));
                Ok(known_as)
            }
            _ => UnknownMeta(format!("Unknown Meta {tyype} {{{:#?}}}", inner_text)).into(),
        }
    }

    fn from_xml_values(from: &Attribute) -> Vec<C::Holder<DocIdentifier>> {
        String::from_utf8(from.value.to_ascii_lowercase())
            .unwrap()
            .split(',')
            // from_xml_value only called for drafts which can only reference rfcs
            .map(|x| C::Holder::from(format!("rfc{x}")))
            .collect()
    }

    pub fn from_xml(attr: &Attribute) -> Result<Meta<C>> {
        match attr.key {
            b"updates" => Ok(Meta::Updates(Self::from_xml_values(attr))),
            b"obsoletes" => Ok(Meta::Obsoletes(Self::from_xml_values(attr))),
            b"replaces" => Ok(Meta::Replaces(Self::from_xml_values(attr).remove(0))),
            _ => UnknownMeta(format!(
                "Unknown Meta {:?} {{{:#?}}}",
                String::from_utf8(attr.key.to_ascii_lowercase()),
                String::from_utf8(attr.value.to_ascii_lowercase())
            ))
            .into(),
        }
    }
}
