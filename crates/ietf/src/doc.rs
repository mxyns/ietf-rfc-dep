use crate::error::{DocError::*, Result};
use crate::meta::Meta;
use crate::url::SourceUrl;
use crate::IdContainer;
use fast_xml::events::Event;
use fast_xml::Reader;
use rayon::iter::Either;
use rayon::prelude::*;
use regex::bytes::Regex;
use reqwest::blocking::Response;
use reqwest::StatusCode;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

/* Identify IETF documents by String (internal name) for now */
pub type DocIdentifier = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
// C represents the container type used to hold document references
pub struct IetfDoc<C>
where
    C: IdContainer,
{
    pub summary: Summary,
    pub meta: Vec<Meta<C>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub id: String,
    pub revision: String,
    pub is_rfc: bool,
    pub url: SourceUrl,
    pub title: String,
}

pub fn name_to_id(name: impl Into<String>) -> DocIdentifier {
    name.into().replace(' ', "").to_lowercase()
}

fn http_get<T: reqwest::IntoUrl + Display>(url: T) -> Result<Response> {
    let resp = reqwest::blocking::get(url)?;
    let status_code = resp.status();
    if !StatusCode::is_success(&status_code) {
        return Query(format!("Error querying {}: {}", resp.url(), status_code)).into();
    }

    Ok(resp)
}

// TODO better api
impl<C> IetfDoc<C>
where
    C: IdContainer,
{
    pub fn id_to_url(id: &DocIdentifier) -> Result<SourceUrl> {
        SourceUrl::new(id)
    }

    pub fn from_name(name: impl Into<String>) -> Result<IetfDoc<C>> {
        let source = Self::id_to_url(&name.into())?;
        IetfDoc::from_html(Either::Left(&source))
    }

    pub fn from_summary(summary: Summary) -> Result<IetfDoc<C>> {
        IetfDoc::from_html(Either::Right(summary))
    }

    fn xml_is_available(document: &Html) -> bool {
        let selector = Selector::parse("div.buttonlist a").unwrap();
        for button in document.select(&selector) {
            if button.text().any(|text| text.eq("xml")) {
                return true;
            }
        }

        false
    }

    fn from_html(source: Either<&SourceUrl, Summary>) -> Result<IetfDoc<C>> {
        let (url, summary_provided) = match source {
            Either::Left(url) => (url.html(), false),
            Either::Right(ref summary) => (summary.url.html(), true),
        };

        let resp = http_get(url.as_str())?;
        if resp.url().path() == "/doc/search" {
            return Query(format!("Error querying {}: document doesn't exist", url)).into();
        }

        let text = resp.text()?;
        let document = Html::parse_document(&text);

        // Find Document Title and Name
        let summary = if !summary_provided {
            let selector = scraper::Selector::parse("#content > h1").unwrap();
            let title_elem = document.select(&selector).next().unwrap();
            let title_text = title_elem.text().collect::<String>();
            let title_regex = Regex::new(r"^\s+(.+)\s+(.+)\s$").unwrap();
            let title_captures = title_regex.captures(title_text.as_ref()).unwrap();
            let title =
                String::from_utf8(title_captures.get(1).unwrap().as_bytes().to_vec()).unwrap();
            let name =
                String::from_utf8(title_captures.get(2).unwrap().as_bytes().to_vec()).unwrap();
            let id = name_to_id(name);

            let is_rfc = id.starts_with("rfc");
            let revision = if is_rfc {
                let selector =
                    scraper::Selector::parse(".revision-list li.page-item:not(.rfc)").unwrap();
                document.select(&selector).last()
            } else {
                let selector =
                    scraper::Selector::parse(".revision-list li.page-item.active").unwrap();
                document.select(&selector).next()
            }
            .map(|x| x.text().map(str::trim).collect::<String>())
            .unwrap_or("00".to_string());

            Some(Summary {
                url: Self::id_to_url(&id)?,
                id, // includes revision (for drafts)
                revision,
                is_rfc,
                title,
            })
        } else {
            None
        };
        let summary = summary.unwrap_or_else(|| source.right().unwrap());

        let doc_meta = if summary.is_rfc || !Self::xml_is_available(&document) {
            Self::parse_meta_html(&document)?
        } else {
            Self::parse_meta_xml(&summary.url)?
        };

        let doc = IetfDoc {
            summary,
            meta: doc_meta,
        };

        Ok(doc)
    }

    fn parse_meta_html(document: &Html) -> Result<Vec<Meta<C>>> {
        let row_selector =
            Selector::parse("#content > table > tbody.meta.align-top.border-top > tr").unwrap();
        let row_name_selector = Selector::parse("th:last-of-type").unwrap();
        let row_value_selector = Selector::parse("td:not(.edit):last-of-type").unwrap();
        let meta_elems = document.select(&row_selector).collect::<Vec<_>>();
        let mut doc_meta: Vec<Meta<C>> = Vec::new();
        for row in meta_elems {
            let name = row
                .select(&row_name_selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>();
            let name = name.trim();
            let value = row.select(&row_value_selector).next().unwrap();

            let metas: Vec<Result<Meta<C>>> = match name {
                "Type" => value
                    .select(&Selector::parse("div").unwrap())
                    .filter_map(|div| {
                        let text: Vec<_> = div.text().collect();
                        if !text.is_empty() {
                            let tyype = text[0].trim().to_lowercase().replace(' ', "_");
                            Some(Meta::from_html(tyype, text))
                        } else {
                            None
                        }
                    })
                    .collect(),
                "Replaces" => {
                    let replaces = value
                        .text()
                        .map(|x| x.trim())
                        .filter(|x| !x.is_empty())
                        .collect();
                    vec![Meta::from_html("replaces".to_string(), replaces)]
                }
                "Replaced by" => {
                    let replaced_by = value
                        .text()
                        .map(|x| x.trim())
                        .filter(|x| !x.is_empty())
                        .collect();
                    vec![Meta::from_html("replaced_by".to_string(), replaced_by)]
                }
                _ => {
                    vec![Err(UnknownMeta(name.to_string()))]
                }
            };

            for meta in metas {
                if let Ok(meta) = meta {
                    doc_meta.push(meta);
                } else {
                    println!("Error: {}", meta.err().unwrap())
                }
            }
        }

        Ok(doc_meta)
    }

    // used only on drafts to get the metas
    fn parse_meta_xml(url: &SourceUrl) -> Result<Vec<Meta<C>>> {
        let resp = http_get(url.xml().clone())?;
        let bytes = resp.bytes()?;
        let mut xml = Reader::from_bytes(bytes.as_ref());
        let mut buf = Vec::new();
        let mut metas: Vec<Meta<C>> = Vec::new();

        loop {
            match xml.read_event(&mut buf) {
                Ok(Event::Start(ref e)) if e.name() == b"rfc" => {
                    for attribute in e.attributes() {
                        match attribute {
                            Ok(ref a) => {
                                let meta = Meta::from_xml(a);
                                if let Ok(meta) = meta {
                                    metas.push(meta);
                                } else {
                                    println!("{}", meta.err().unwrap())
                                }
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    }
                    break;
                }
                Ok(Event::Eof) => {
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                }
            }
            buf.clear();
        }

        Ok(metas)
    }

    pub fn lookup(title: &str, limit: usize, include_drafts: bool) -> Result<Vec<Summary>> {
        if title.is_empty() {
            return Lookup("no query".to_string()).into();
        }

        let rfc_only = if include_drafts { "" } else { "&states__in=3" };
        let query = format!("https://datatracker.ietf.org/api/v1/doc/document/?title__icontains={title}&limit={limit}&offset=0&format=json{rfc_only}&type__in=draft");

        println!("query = {query}");
        let resp = reqwest::blocking::get(query);
        let resp = if let Ok(resp) = resp {
            resp
        } else {
            return Lookup(format!("could not http/GET {}", resp.err().unwrap())).into();
        };

        let status_code = &resp.status();
        if !StatusCode::is_success(status_code) {
            return Lookup(format!(
                "unsuccessful status http/GET status {}",
                status_code
            ))
            .into();
        }

        let summaries: Vec<Summary> = resp
            .json::<serde_json::Value>()
            .unwrap()
            .get_mut("objects")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .par_drain(..)
            .map(|obj| {
                let rfc_num = obj.get("rfc");
                let revision = obj.get("rev").unwrap().as_str().unwrap().to_string();
                let id = if rfc_num.is_some_and(|val| !val.is_null()) {
                    format!("rfc{}", rfc_num.unwrap().as_str().unwrap())
                } else {
                    format!(
                        "{}-{}",
                        obj.get("name").unwrap().as_str().unwrap(),
                        revision
                    )
                };

                Summary {
                    url: Self::id_to_url(&id).unwrap(),
                    id,
                    revision,
                    title: obj.get("title").unwrap().as_str().unwrap().to_string(),
                    is_rfc: rfc_num.is_some(),
                }
            })
            .collect();

        println!("{} matches = {:#?}", summaries.len(), &summaries);

        Ok(summaries)
    }

    pub fn meta_count(&self) -> usize {
        let mut len = 0;
        for meta in &self.meta {
            len += match meta {
                Meta::Updates(list)
                | Meta::Obsoletes(list)
                | Meta::UpdatedBy(list)
                | Meta::ObsoletedBy(list) => list.len(),
                Meta::Was(_) | Meta::Replaces(_) | Meta::ReplacedBy(_) | Meta::AlsoKnownAs(_) => 1,
            }
        }

        len
    }
}
