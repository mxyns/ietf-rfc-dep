use regex;
use regex::bytes::Regex;
use crate::cache::{CachedDoc};

#[derive(Debug)]
pub struct IetfDoc {
    pub name: String,
    pub url: String,
    pub title: Option<String>,
    pub meta: Option<Vec<Meta>>,
}

#[derive(Debug)]
pub enum DocRef {
    Identifier(String),
    CacheEntry(CachedDoc),
}

#[derive(Debug)]
pub enum Meta {
    Updates(Vec<DocRef>),
    Obsoletes(Vec<DocRef>),
    UpdatedBy(Vec<DocRef>),
    Was(String),
}

impl Meta {
    fn from_html(tyype: String, inner_text: Vec<&str>) -> Result<Meta, String> {
        match tyype.as_str() {
            "updated_by" => {
                let updaters = Meta::UpdatedBy(Self::meta_array_to_doc_identifiers(inner_text));
                Ok(updaters)
            }
            "obsoletes" => {
                let obsoleted = Meta::Obsoletes(Self::meta_array_to_doc_identifiers(inner_text));
                Ok(obsoleted)
            }
            "updates" => {
                let updated = Meta::Updates(Self::meta_array_to_doc_identifiers(inner_text));
                Ok(updated)
            }
            "was" => {
                let was = Meta::Was(inner_text[1].trim().to_string());
                Ok(was)
            }
            _ => {
                Err("Unknown Type".to_string())
            }
        }
    }

    fn meta_array_to_doc_identifiers(lines: Vec<&str>) -> Vec<DocRef> {
        lines.into_iter().skip(1).step_by(2).map(|x| {
            DocRef::Identifier(name_to_id(x))
        }).collect()
    }
}

pub fn name_to_id(name: &str) -> String {
    name.to_string().replace(" ", "").to_lowercase()
}

impl IetfDoc {

    pub fn from_url(url: &str) -> IetfDoc {

        let resp = reqwest::blocking::get(url).unwrap();
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


            if let Ok(meta) = Meta::from_html(tyype, inner_text) {
                doc_meta.push(meta);
            }
        }

        let doc = IetfDoc {
            name: name_to_id(name.as_str()),
            url: url.to_string(),
            title: Some(title),
            meta: Some(doc_meta),
        };

        doc
    }
}