use std::collections::HashSet;
use regex;
use regex::bytes::Regex;
use serde::{Deserialize, Serialize};
use crate::cache::{CacheReference, RelationalEntry, ResolvableEntry};

/* Identify IETF documents by String (internal name) for now */
pub type DocIdentifier = String;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IetfDoc {
    pub name: String,
    pub url: String,
    pub title: String,
    pub meta: Vec<Meta>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum Meta {
    #[default]
    None,
    Updates(Vec<CacheReference<DocIdentifier>>),
    UpdatedBy(Vec<CacheReference<DocIdentifier>>),
    Obsoletes(Vec<CacheReference<DocIdentifier>>),
    ObsoletedBy(Vec<CacheReference<DocIdentifier>>),
    Was(DocIdentifier),
}

impl Meta {
    fn from_html(tyype: String, inner_text: Vec<&str>) -> Result<Meta, String> {
        match tyype.as_str() {
            "updated_by" => {
                let updaters = Meta::UpdatedBy(Self::meta_array_to_doc_identifiers(inner_text));
                Ok(updaters)
            }
            "updates" => {
                let updated = Meta::Updates(Self::meta_array_to_doc_identifiers(inner_text));
                Ok(updated)
            }
            "obsoletes" => {
                let obsoleted = Meta::Obsoletes(Self::meta_array_to_doc_identifiers(inner_text));
                Ok(obsoleted)
            }
            "obsoleted_by" => {
                let obsoleters = Meta::ObsoletedBy(Self::meta_array_to_doc_identifiers(inner_text));
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

    fn meta_array_to_doc_identifiers(lines: Vec<&str>) -> Vec<CacheReference<DocIdentifier>> {
        lines.into_iter().skip(1).step_by(2).map(|x| {
            CacheReference::Unknown(name_to_id(x))
        }).collect()
    }
}

pub fn name_to_id(name: &str) -> DocIdentifier {
    name.to_string().replace(" ", "").to_lowercase()
}

impl IetfDoc {
    pub fn from_url(url: String) -> IetfDoc {
        let resp = reqwest::blocking::get(&*url).unwrap();
        let text = resp.text().unwrap();
        let document = scraper::Html::parse_document(&text);

        // Find Document Title and Name
        let selector = scraper::Selector::parse("#content > h1").unwrap();
        let title_elem = document.select(&selector).next().unwrap();
        let title_text = title_elem.text().collect::<String>();
        let title_regex = Regex::new(r"^\s+(.+)\s+(.+)\s$").unwrap();
        let title_captures = title_regex.captures(title_text.as_ref()).unwrap();
        let title = String::from_utf8(title_captures.get(1).unwrap().as_bytes().to_vec()).unwrap();
        let name = String::from_utf8(title_captures.get(2).unwrap().as_bytes().to_vec()).unwrap();

        // Find Document Relationship Metadata
        let selector = scraper::Selector::parse("#content > table > tbody.meta.align-top.border-top > tr:nth-child(1) > td:nth-child(4) > div").unwrap();
        let meta_elems = document.select(&selector).collect::<Vec<_>>();

        // Parse Document Relationship Metadata
        let mut doc_meta: Vec<Meta> = Vec::new();
        for item in meta_elems {
            let inner_text = item.text().collect::<Vec<_>>();
            // Skip empty items
            if inner_text.len() == 0 {
                continue;
            }

            // Extract type from Html innerText
            let tyype = inner_text[0].trim().to_lowercase();
            let regex = Regex::new(r"\s").unwrap();
            let tyype = regex.replace_all(tyype.as_bytes(), "_".as_bytes()).to_vec();
            let tyype = String::from_utf8(tyype).unwrap();


            let meta = Meta::from_html(tyype, inner_text);
            if let Ok(meta) = meta {
                doc_meta.push(meta);
            } else {
                println!("Meta: {}", meta.err().unwrap())
            }
        }

        let doc = IetfDoc {
            name: name_to_id(name.as_str()),
            url: url.to_string(),
            title,
            meta: doc_meta,
        };

        doc
    }

    pub fn lookup(title: &str) -> Vec<IetfDoc> {
        if title.len() == 0 {
            return vec![];
        }

        let query = format!("https://datatracker.ietf.org/api/v1/doc/document/?title__icontains={title}&limit=100&offset=0&name__startswith=rfc&format=json");
        println!("query = {query}");
        let resp = reqwest::blocking::get(query).unwrap();
        let json: serde_json::Value = resp.json().unwrap();

        let urls: Vec<String> = json.get("objects").unwrap().as_array().unwrap().iter()
            .map(|obj| obj.get("name"))
            .flatten()
            .map(serde_json::Value::as_str)
            .flatten()
            .map(|name| format!("https://datatracker.ietf.org/doc/{name}"))
            .collect();

        println!("{} matches = {:#?}", urls.len(), &urls);

        urls.into_iter()
            .map(IetfDoc::from_url)
            .collect()
    }

    pub fn missing(&self) -> usize {
        let mut missing = 0;
        for meta in &self.meta {
            match meta {
                Meta::Updates(list)
                | Meta::Obsoletes(list)
                | Meta::UpdatedBy(list)
                | Meta::ObsoletedBy(list) => {
                    for item in list {
                        match item {
                            CacheReference::Unknown(_) => {
                                missing += 1;
                            }
                            CacheReference::Cached(_) => {}
                        };
                    };
                }
                Meta::Was(_) | Meta::None => {}
            }
        };

        missing
    }
}

// Implement resolve dependency algorithms when value is IetfDoc
impl RelationalEntry<DocIdentifier> for IetfDoc {
    fn get_relations(&self) -> HashSet<DocIdentifier> {
        let mut to_update = HashSet::new();
        for meta in &self.meta {
            match meta {
                Meta::Updates(list)
                | Meta::Obsoletes(list)
                | Meta::UpdatedBy(list)
                | Meta::ObsoletedBy(list) => {
                    for item in list {
                        match item {
                            CacheReference::Unknown(id) => {
                                to_update.insert(id.clone());
                            }
                            CacheReference::Cached(_) => {}
                        };
                    };
                }
                Meta::Was(_) | Meta::None => {}
            }
        }

        to_update
    }

    fn update_reference(&mut self, _id: &DocIdentifier, is_known: impl Fn(&DocIdentifier) -> bool) {
        for meta in &mut self.meta {
            match meta {
                Meta::Updates(list)
                | Meta::Obsoletes(list)
                | Meta::UpdatedBy(list)
                | Meta::ObsoletedBy(list) => {
                    for item in list {
                        let (CacheReference::Cached(ref_id) | CacheReference::Unknown(ref_id)) = item.clone();
                        if is_known(&ref_id) {
                            *item = CacheReference::Cached(ref_id);
                        } else {
                            *item = CacheReference::Unknown(ref_id);
                        }
                    };
                }
                Meta::Was(_) | Meta::None => {}
            }
        }
    }
}

impl ResolvableEntry<DocIdentifier> for IetfDoc {
    fn get_value(id: DocIdentifier) -> Self {
        IetfDoc::from_url(format!("https://datatracker.ietf.org/doc/{}", id))
    }
}