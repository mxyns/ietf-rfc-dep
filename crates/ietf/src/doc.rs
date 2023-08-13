use crate::meta::Meta;
use crate::IdContainer;
use rayon::prelude::*;
use regex::bytes::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/* Identify IETF documents by String (internal name) for now */
pub type DocIdentifier = String;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
// C represents the container type used to hold document references
pub struct IetfDoc<C>
where
    C: IdContainer,
{
    pub name: String,
    pub url: String,
    pub title: String,
    pub meta: Vec<Meta<C>>,
}

pub fn name_to_id(name: &str) -> DocIdentifier {
    name.to_string().replace(' ', "").to_lowercase()
}

impl<C> IetfDoc<C>
where
    C: IdContainer,
{
    pub fn from_url(url: String) -> IetfDoc<C> {
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
        let mut doc_meta: Vec<Meta<C>> = Vec::new();
        for item in meta_elems {
            let inner_text = item.text().collect::<Vec<_>>();
            // Skip empty items
            if inner_text.is_empty() {
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

        let selector = scraper::Selector::parse(
            "tbody.meta:nth-child(1) > tr:nth-child(4) > td:nth-child(4) > a:nth-child(1)",
        )
        .unwrap();
        if let Some(replaces) = document
            .select(&selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>())
        {
            let meta = Meta::from_html("replaces".to_string(), replaces);
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

    pub fn lookup(title: &str, limit: usize, rfc_only: bool) -> Vec<IetfDoc<C>> {
        if title.is_empty() {
            return vec![];
        }

        let rfc_only = if rfc_only { "&states__in=3" } else { "" };
        let query = format!("https://datatracker.ietf.org/api/v1/doc/document/?title__icontains={title}&limit={limit}&offset=0&format=json{rfc_only}&type__in=draft");

        println!("query = {query}");
        let resp = reqwest::blocking::get(query).unwrap();
        let json: serde_json::Value = resp.json().unwrap();

        let urls: Vec<String> = json
            .get("objects")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|obj| obj.get("name"))
            .filter_map(serde_json::Value::as_str)
            .map(|name| format!("https://datatracker.ietf.org/doc/{name}"))
            .collect();

        println!("{} matches = {:#?}", urls.len(), &urls);

        urls.into_par_iter().map(IetfDoc::from_url).collect()
    }

    pub fn meta_count(&self) -> usize {
        let mut len = 0;
        for meta in &self.meta {
            match meta {
                Meta::Updates(list)
                | Meta::Obsoletes(list)
                | Meta::UpdatedBy(list)
                | Meta::ObsoletedBy(list) => {
                    len += list.len();
                }
                Meta::Was(_) | Meta::Replaces(_) => len += 1,
            }
        }

        len
    }
}